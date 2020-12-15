#![feature(btree_drain_filter, map_first_last)]
#[macro_use]
extern crate clap;
extern crate flate2;
extern crate glob;

use clap::{App, AppSettings};
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use glob::glob;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
#[cfg(target_family = "windows")]
use std::os::windows::fs::symlink_file as symlink;
use std::result::Result;

const BUFFER_SIZE: usize = 32 * 1024;

fn none_err() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "NoneError")
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new(crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .author(crate_authors!(";"))
        .about(crate_description!())
        .args_from_usage(
            "
            <read1> -o1=[FILE] 'read1 output path of concated pair'
            <read2> -o2=[FILE] 'read2 output path of concated pair'
            <srcdir> 'fastq pair source directory to concat, fastq glob `*_R[12].fastq.gz` or `*_R[12].fq.gz`'
            ",
        )
        .get_matches();
    let srcdir: &str = args.value_of("srcdir").ok_or_else(none_err)?;
    let read1: &str = args.value_of("read1").ok_or_else(none_err)?;
    let read2: &str = args.value_of("read2").ok_or_else(none_err)?;
    let mut collector: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    // *_R[12].f{ast,}q.gz
    for alias in &["fastq", "fq"] {
        let slen = 7 + alias.len();
        for i in 1..=2 {
            for path in glob(&format!("{}/*_R{}.{}.gz", srcdir, i, alias))? {
                let path: String = path?.to_string_lossy().to_string();
                let prefix: String = path.clone()[..(path.len() - slen)]
                    .rsplit('/')
                    .next()
                    .ok_or_else(none_err)?
                    .to_owned();
                collector
                    .entry(prefix)
                    .or_insert_with(BTreeMap::new)
                    .entry(format!("read{}", i))
                    .or_insert(path);
            }
        }
        // remove singles
        collector.drain_filter(|_, v| v.len() <= 1);
    }
    if collector.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Gzipped fastq not found").into());
    } else if collector.len() == 1 {
        let (_, pair) = collector.first_key_value().ok_or_else(none_err)?;
        fs::remove_file(read1).unwrap_or_else(|e| eprintln!("{}", e));
        symlink(&pair[&"read1".to_owned()], read1)?;
        fs::remove_file(read2).unwrap_or_else(|e| eprintln!("{}", e));
        symlink(&pair[&"read2".to_owned()], read2)?;
        return Ok(());
    }
    let mut stream1 = BufWriter::new(GzEncoder::new(File::create(read1)?, Compression::fast()));
    let mut stream2 = BufWriter::new(GzEncoder::new(File::create(read2)?, Compression::fast()));
    for i in collector.values() {
        let mut buffer: Vec<u8> = vec![0; BUFFER_SIZE];
        println!("Concating {} ...", &i[&"read1".to_owned()]);
        let mut reader = MultiGzDecoder::new(File::open(&i[&"read1".to_owned()])?);
        let mut size = BUFFER_SIZE;
        while size != 0 {
            size = reader.read(&mut buffer)?;
            stream1.write_all(&buffer[..size])?;
        }
        println!("Concating {} ...", &i[&"read2".to_owned()]);
        reader = MultiGzDecoder::new(File::open(&i[&"read2".to_owned()])?);
        size = BUFFER_SIZE;
        while size != 0 {
            size = reader.read(&mut buffer)?;
            stream2.write_all(&buffer[..size])?;
        }
    }
    stream1.into_inner()?.try_finish()?;
    stream2.into_inner()?.try_finish()?;
    Ok(())
}
