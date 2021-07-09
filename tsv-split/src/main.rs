#[macro_use]
extern crate clap;
extern crate csv;

use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;

use clap::{AppSettings, Clap};
use csv::ReaderBuilder as CsvReaderBuilder;
use csv::Writer as CsvWriter;
use csv::WriterBuilder as CsvWriterBuilder;

type Row = BTreeMap<String, String>;

#[derive(Clap)]
#[clap(name = crate_name!(), version = crate_version!(), author = crate_authors!(), about = crate_description!())]
#[clap(setting = AppSettings::ArgRequiredElseHelp)]
struct Opts {
    #[clap(short, about = "Output directory.")]
    outdir: String,
    #[clap(long, about = "Split on which header.")]
    on: String,
    #[clap(about = "Input file path.")]
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let input = PathBuf::from(&opts.input);
    let prefix: &str = input.file_stem().unwrap().to_str().unwrap();
    let ext: &str = input
        .extension()
        .unwrap_or(OsStr::new("tsv"))
        .to_str()
        .unwrap();
    let mut reader = CsvReaderBuilder::new()
        .delimiter(b'\t')
        .buffer_capacity(1 << 20)
        .has_headers(true)
        .from_path(&opts.input)?;
    let mut writers: HashMap<String, CsvWriter<File>> = HashMap::new();
    let headers = reader.headers()?.to_owned();
    for rec in reader.deserialize() {
        let rec: Row = rec?;
        let part: String = rec
            .get(&opts.on)
            .expect(&format!("No header named {}.", &opts.on))
            .to_owned();
        let writer = writers.entry(part.clone()).or_insert_with(|| {
            let mut nw = CsvWriterBuilder::new()
                .delimiter(b'\t')
                .has_headers(true)
                .buffer_capacity(1 << 22)
                .from_path(format!("{}/{}.{}.{}", &opts.outdir, prefix, part, ext))
                .unwrap();
            nw.write_record(&headers).unwrap();
            nw
        });
        writer.write_record(headers.iter().map(|v| &rec[v]).collect::<Vec<&String>>())?;
    }
    for i in writers.values_mut() {
        i.flush()?;
    }
    Ok(())
}
