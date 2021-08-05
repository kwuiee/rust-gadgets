#[macro_use]
extern crate clap;
extern crate csv;

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use clap::{AppSettings, Clap};
use csv::ReaderBuilder as CsvReaderBuilder;
use csv::StringRecord;
use csv::Writer as CsvWriter;
use csv::WriterBuilder as CsvWriterBuilder;

const BUFFER: usize = 1 << 20;
type Row = BTreeMap<String, String>;

struct WriterGroup<'a, 'b, 'c> {
    prefix: &'a str,
    ext: &'b str,
    headers: &'c StringRecord,
    limit: Option<u32>,
    inner: HashMap<String, (CsvWriter<File>, u32, u32)>,
}

impl<'a, 'b, 'c> WriterGroup<'a, 'b, 'c> {
    fn new(prefix: &'a str, ext: &'b str, headers: &'c StringRecord, limit: Option<u32>) -> Self {
        Self {
            prefix,
            ext,
            headers,
            limit,
            inner: HashMap::new(),
        }
    }

    fn write(&mut self, part: String, rec: &Vec<&String>) -> Result<(), io::Error> {
        // If line number reached limit
        match (self.limit, self.inner.get_mut(&part)) {
            (Some(num), Some(col)) => {
                if num <= col.1 {
                    col.2 += 1;
                    col.0 = CsvWriterBuilder::new()
                        .delimiter(b'\t')
                        .has_headers(true)
                        .buffer_capacity(BUFFER)
                        .from_path(format!("{}.{}.{}.{}", self.prefix, &part, col.2, self.ext))?;
                    col.0.write_record(self.headers)?;
                    col.1 = 1;
                };
                col.0.write_record(rec)?;
                col.1 += 1;
            }
            (Some(_), None) => {
                let tag = 0;
                let mut wtr = CsvWriterBuilder::new()
                    .delimiter(b'\t')
                    .has_headers(true)
                    .buffer_capacity(BUFFER)
                    .from_path(format!("{}.{}.{}.{}", self.prefix, &part, tag, self.ext))?;
                wtr.write_record(self.headers)?;
                wtr.write_record(rec)?;
                self.inner.entry(part).or_insert((wtr, 2, tag));
            }
            (None, Some((wtr, _, _))) => {
                wtr.write_record(rec)?;
            }
            (None, None) => {
                let mut wtr = CsvWriterBuilder::new()
                    .delimiter(b'\t')
                    .has_headers(true)
                    .buffer_capacity(BUFFER)
                    .from_path(format!("{}.{}.{}", self.prefix, &part, self.ext))?;
                wtr.write_record(self.headers)?;
                wtr.write_record(rec)?;
                self.inner.entry(part).or_insert((wtr, 0, 0));
            }
        };
        Ok(())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        for i in self.inner.values_mut() {
            i.0.flush()?;
        }
        Ok(())
    }
}

#[derive(Clap)]
#[clap(name = crate_name!(), version = crate_version!(), author = crate_authors!(), about = crate_description!())]
#[clap(setting = AppSettings::ArgRequiredElseHelp)]
struct Opts {
    #[clap(short, about = "Output directory.")]
    outdir: String,
    #[clap(long, about = "Split on header.")]
    header: String,
    #[clap(long, about = "Split on row count.")]
    limit: Option<u32>,
    #[clap(about = "Input file path.")]
    input: String,
}

fn main() -> Result<(), io::Error> {
    let opts = Opts::parse();
    let input = PathBuf::from(&opts.input);
    let prefix: &str = &format!(
        "{}/{}",
        opts.outdir,
        input.file_stem().unwrap().to_str().unwrap()
    );
    let ext: &str = input
        .extension()
        .unwrap_or(OsStr::new("tsv"))
        .to_str()
        .unwrap();
    let mut reader = CsvReaderBuilder::new()
        .delimiter(b'\t')
        .buffer_capacity(BUFFER)
        .has_headers(true)
        .from_path(&opts.input)?;
    let headers = reader.headers()?.to_owned();
    let mut writers = WriterGroup::new(prefix, ext, &headers, opts.limit);
    for rec in reader.deserialize() {
        let rec: Row = rec?;
        let part: String = rec
            .get(&opts.header)
            .expect(&format!("No header named {}.", &opts.header))
            .to_owned();
        let recw = headers.iter().map(|v| &rec[v]).collect::<Vec<&String>>();
        writers.write(part, &recw)?;
    }
    writers.flush().into()
}
