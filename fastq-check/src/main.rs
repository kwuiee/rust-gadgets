#![allow(deprecated)]

extern crate bio;
#[macro_use]
extern crate clap;
extern crate flate2;
extern crate serde;
extern crate serde_json;

use bio::io::fastq::{FastqRead, Reader, Record};
use clap::{App, AppSettings};
use flate2::read::MultiGzDecoder;
use serde::Serialize;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::process;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

// check points
// 1. read id first half be the same for pair
// 2. every four line a read id
// 3. quality length equal to sequence length

#[derive(Debug, Serialize)]
struct Criteria {
    base_number: u64,
    read1_base_number: u64,
    read2_base_number: u64,
    pair_readed: u32,
    c_reads: bool,
    c_base: bool,
    c_format: bool,
    summary: bool,
}

impl Criteria {
    fn new() -> Self {
        Self {
            base_number: 0u64,
            read1_base_number: 0u64,
            read2_base_number: 0u64,
            pair_readed: 0u32,
            c_reads: false,
            c_base: false,
            c_format: false,
            summary: false,
        }
    }

    fn sum(&mut self) {
        self.summary = self.c_reads && self.c_base && self.c_format
    }

    fn set_reads_when(&mut self, num: u32) {
        self.c_reads = self.pair_readed >= num
    }

    fn set_base_when(self: &mut Self, num: u64) {
        self.c_base = self.base_number >= num
    }

    fn set_format(self: &mut Self, state: bool) {
        self.c_format = state
    }
}

fn from_pair_stream<T: Read>(read1: T, read2: T, head: u32) -> Criteria {
    let mut summary = Criteria::new();

    let mut fq1 = Reader::new(read1);
    let mut fq2 = Reader::new(read2);
    let mut rec1 = Record::new();
    let mut rec2 = Record::new();
    let mut counter: u32 = 0;

    match loop {
        if head > 0 && counter >= head {
            break Ok(());
        }
        // id starts with @, 4 line per read
        match fq1.read(&mut rec1).and(fq2.read(&mut rec2)) {
            Ok(_) => (),
            e => break e,
        };
        // one reached end while the other not
        if rec1.is_empty() && rec2.is_empty() {
            break Ok(());
        } else if rec1.is_empty() || rec2.is_empty() {
            break Err(Error::new(
                ErrorKind::InvalidData,
                "one of fastq read to end while the other not",
            ));
        };
        counter += 1;
        // check id().is_empty, seq.is_ascii(), qual.is_ascii(), seq().len() != qual().len()
        match rec1.check().and(rec2.check()) {
            Ok(_) => (),
            Err(e) => break Err(Error::new(ErrorKind::InvalidData, e)),
        };
        // check read1 id == read2 id
        if rec1.id() != rec2.id() {
            break Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "read1 and read2 id not same: read1: {}, read2: {}",
                    rec1.id(),
                    rec2.id()
                ),
            ));
        };
        summary.pair_readed += 1;
        summary.read1_base_number += rec1.seq().len() as u64;
        summary.read2_base_number += rec2.seq().len() as u64;
    } {
        Ok(_) => summary.set_format(true),
        Err(e) => eprintln!("{}", e),
    };

    summary.base_number = summary.read1_base_number + summary.read2_base_number;
    summary
}

fn from_single_stream<T: Read>(read1: T, head: u32) -> Criteria {
    let mut summary = Criteria::new();

    let mut fq1 = Reader::new(read1);
    let mut rec1 = Record::new();
    let mut counter: u32 = 0;

    match loop {
        if head > 0 && counter >= head {
            break Ok(());
        }
        // id starts with @, 4 line per read
        match fq1.read(&mut rec1) {
            Ok(()) => (),
            e => break e,
        };
        // one reached end while the other not
        if rec1.is_empty() {
            break Ok(());
        };
        counter += 1;
        // check id().is_empty, seq.is_ascii(), qual.is_ascii(), seq().len() != qual().len()
        match rec1.check() {
            Ok(_) => (),
            Err(e) => break Err(Error::new(ErrorKind::InvalidData, e)),
        };
        summary.pair_readed += 1;
        summary.read1_base_number += rec1.seq().len() as u64;
    } {
        Ok(_) => summary.set_format(true),
        Err(e) => eprintln!("{}", e),
    };

    summary.base_number = summary.read1_base_number;
    summary
}

fn run(
    read1path: &str,
    read2path: &str,
    head: u32,
    base_limit: u64,
    read_limit: u32,
) -> Result<Criteria, Error> {
    let read1ext: &str = read1path.split('.').last().unwrap();
    let read1r: Box<dyn Read> = if read1ext != "gz" {
        Box::new(File::open(read1path)?)
    } else {
        Box::new(MultiGzDecoder::new(File::open(read1path)?))
    };
    let read2ext: &str = read2path.split('.').last().unwrap();
    let read2r: Box<dyn Read> = if read2ext != "gz" {
        Box::new(File::open(read2path)?)
    } else {
        Box::new(MultiGzDecoder::new(File::open(read2path)?))
    };
    let mut summary = if read1path == read2path {
        from_single_stream(read1r, head)
    } else {
        from_pair_stream(read1r, read2r, head)
    };
    summary.set_reads_when(read_limit);
    summary.set_base_when(base_limit);
    summary.sum();
    Ok(summary)
}

fn main() {
    let args = App::new(crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .version(crate_version!())
        .author(crate_authors!(";"))
        .about(crate_description!())
        .args_from_usage(
            "
            <read1> -1, --read1=[FILE] 'first read of a pair'
            [read2] -2, --read2=[FILE] 'second read of a pair'
            [head] -n, --head=[NUMBER] 'only check first n reads'
            [base] -b, --base=[NUMBER] 'min base number threshold, default 0'
            [reads] -r, --reads=[NUMBER] 'min reads number threshold, default 0'
            ",
        )
        .get_matches();
    let head: u32 = args
        .value_of("head")
        .unwrap_or("0")
        .parse()
        .expect("invalid input value for arg head");
    let base: u64 = args
        .value_of("base")
        .unwrap_or("0")
        .parse()
        .expect("invalid input value for arg head");
    let reads: u32 = args
        .value_of("reads")
        .unwrap_or("0")
        .parse()
        .expect("invalid input value for arg head");
    let read1path: &str = args.value_of("read1").unwrap();
    let read2path: &str = args.value_of("read2").unwrap_or(read1path);
    match run(read1path, read2path, head, base, reads) {
        Ok(v) => {
            println!("{}", serde_json::to_string_pretty(&v).unwrap());
            process::exit(1 - v.summary as i32)
        }
        Err(e) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&Criteria::new()).unwrap()
            );
            eprintln!("{}", e);
            process::exit(1)
        }
    };
}
