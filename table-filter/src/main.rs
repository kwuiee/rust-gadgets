#[macro_use]
extern crate clap;
extern crate csv;
extern crate regex;

use clap::{App, AppSettings, Arg};
use csv::{Reader, ReaderBuilder, Writer, WriterBuilder};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::iter::{IntoIterator, Iterator};

#[derive(Debug)]
struct Filter {
    include: HashMap<String, Vec<Regex>>,
    exclude: HashMap<String, Vec<Regex>>,
    less: bool,
}

impl<'a> Filter {
    fn new() -> Filter {
        Filter {
            include: HashMap::<String, Vec<Regex>>::new(),
            exclude: HashMap::<String, Vec<Regex>>::new(),
            less: false,
        }
    }

    fn mode(&mut self, m: bool) {
        self.less = m
    }

    fn included(&self, k: &str, v: &str) -> Option<bool> {
        if !self.include.contains_key(k) {
            None
        } else {
            Some(self.include[k].iter().any(|x| x.is_match(v)))
        }
    }

    fn excluded(&self, k: &str, v: &str) -> Option<bool> {
        if !self.exclude.contains_key(k) {
            None
        } else {
            Some(self.exclude[k].iter().any(|x| x.is_match(v)))
        }
    }

    fn hard_exclude(&self, k: &str, v: &str) -> Option<bool> {
        match (self.included(k, v), self.excluded(k, v)) {
            (_, Some(true)) => Some(true),
            (Some(i), _) => Some(!i),
            _ => None,
        }
    }

    fn loose_include(&self, k: &str, v: &str) -> Option<bool> {
        match (self.included(k, v), self.excluded(k, v)) {
            (Some(true), _) => Some(true),
            (_, Some(e)) => Some(!e),
            _ => None,
        }
    }

    fn qualify<I>(&self, i: I) -> bool
    where
        I: Iterator<Item = (String, &'a str)>,
    {
        if self.less {
            let mut num = 0;
            let ex = i.filter_map(|(i, j)| self.hard_exclude(&i, j)).any(|i| {
                num += 1;
                i
            });
            num >= 1 && !ex
        } else {
            let mut num = 0;
            let inc = i.filter_map(|(i, j)| self.loose_include(&i, j)).all(|i| {
                num += 1;
                i
            });
            num <= 0 || inc
        }
    }

    fn add_include(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        Filter::parse_to(s, &mut self.include)
    }

    fn add_exclude(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        Filter::parse_to(s, &mut self.exclude)
    }

    fn parse_to(s: &str, d: &mut HashMap<String, Vec<Regex>>) -> Result<(), Box<dyn Error>> {
        s.split(';')
            .try_for_each::<_, Result<(), Box<dyn Error>>>(|x| {
                let v: Vec<&str> = x.splitn(2, '=').collect();
                d.entry(v[0].to_owned())
                    .or_insert_with(Vec::<Regex>::new)
                    .push(Regex::new(v[1])?);
                Ok(())
            })?;
        Ok(())
    }
}

fn err(n: u32) -> io::Error {
    io::Error::new(
        io::ErrorKind::Other,
        format!("line {} option none error", n),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new(crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .author(crate_authors!(";"))
        .about(crate_description!())
        .args(&[
            Arg::with_name("output")
                .short("o")
                .long("output")
                .default_value("-")
                .help("output file, `-` mean stdout"),
            Arg::with_name("include")
                .short("i")
                .long("include")
                .multiple(true)
                .takes_value(true)
                .number_of_values(1)
                .help("Pattern for value including, e.g. 'header1=(?:regex:pattern);header2=^1$'"),
            Arg::with_name("exclude")
                .short("e")
                .long("exclude")
                .multiple(true)
                .takes_value(true)
                .number_of_values(1)
                .required_unless("include")
                .help("Pattern for value excluding, overrided by `include` if overlapped"),
            Arg::with_name("tab")
                .long("tab")
                .takes_value(false)
                .help("Input is tab separated (default)"),
            Arg::with_name("comma")
                .long("comma")
                .takes_value(false)
                .conflicts_with("tab")
                .help("Input is comma separated"),
            Arg::with_name("noheader")
                .long("no-header")
                .takes_value(false)
                .help("Use 0-N indexed header instead of first row"),
            Arg::with_name("less")
                .long("less")
                .takes_value(false)
                .help("Give less result")
                .long_help(
                    "
1. Output included ones, this will be activate with `less` parameter set
    In this circumstance excluding is prior to including.
    Including pattern must be given, otherwise no result will be output
2. Output all except excluding ones (default)
    In this circumstance including is prior to excluding.
    Excluding pattern must be given, otherwise no filter will be applied
            ",
                ),
            Arg::with_name("src")
                .takes_value(true)
                .multiple(false)
                .default_value("-")
                .help("Input file, `-` mean stdin"),
        ])
        .get_matches();
    let mut filter = Filter::new();
    filter.mode(args.is_present("less"));
    let delimiter = if args.is_present("comma") {
        b','
    } else {
        b'\t'
    };
    if let Some(mut v) = args.values_of("include") {
        v.try_for_each(|x| filter.add_include(x))?
    };
    if let Some(mut v) = args.values_of("exclude") {
        v.try_for_each(|x| filter.add_exclude(x))?
    };
    let mut reader: Reader<Box<dyn Read>> = if let Some("-") = args.value_of("src") {
        ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(!args.is_present("noheader"))
            .from_reader(Box::new(io::stdin()))
    } else if let Some(v) = args.value_of("src") {
        ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(!args.is_present("noheader"))
            .from_reader(Box::new(File::open(v)?))
    } else {
        return Err(err(line!()).into());
    };
    let mut writer: Writer<Box<dyn Write>> = if let Some("-") = args.value_of("output") {
        WriterBuilder::new()
            .delimiter(delimiter)
            .has_headers(!args.is_present("noheader"))
            .from_writer(Box::new(io::stdout()))
    } else if let Some(v) = args.value_of("output") {
        WriterBuilder::new()
            .delimiter(delimiter)
            .has_headers(!args.is_present("noheader"))
            .from_writer(Box::new(File::create(v)?))
    } else {
        return Err(err(line!()).into());
    };
    match reader.has_headers() {
        true => {
            let headers = reader.headers()?.clone();
            writer.write_record(reader.headers()?)?;
            reader
                .into_records()
                .try_for_each::<_, Result<(), Box<dyn Error>>>(|record| {
                    let record = record?;
                    if filter.qualify(headers.iter().map(|i| i.to_owned()).zip(record.into_iter()))
                    {
                        writer.write_record(&record)?;
                    };
                    Ok(())
                })?;
        }
        false => {
            reader
                .into_records()
                .try_for_each::<_, Result<(), Box<dyn Error>>>(|record| {
                    let record = record?;
                    if filter.qualify(
                        record
                            .into_iter()
                            .enumerate()
                            .map(|(i, j)| (i.to_string(), j))
                            .into_iter(),
                    ) {
                        writer.write_record(&record)?;
                    }
                    Ok(())
                })?;
        }
    };
    Ok(())
}
