#![feature(exclusive_range_pattern)]
#[macro_use]
extern crate clap;
extern crate rust_htslib;

use clap::{App, AppSettings};
use rust_htslib::bam::Reader as BamReader;
use rust_htslib::bam::Writer as BamWriter;
use rust_htslib::bam::{
    record::{Aux, Cigar, CigarString, Record},
    Format, Header, Read,
};
use std::cmp;
use std::default::Default;
use std::iter::Iterator;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Default)]
struct Dimension {
    start: u32,
    end: u32,
}

trait EasyCigar {
    fn forward_cigar_string(&self) -> CigarString;
    fn count_sa(&self) -> Result<u16, &'static str>;
    fn first_sa_forward_cigar_string(&self) -> Result<CigarString, &'static str>;
}

impl EasyCigar for Record {
    fn forward_cigar_string(&self) -> CigarString {
        let cigar_iter = self.raw_cigar().iter().map(|&c| {
            let len = c >> 4;
            match c & 0b1111 {
                0 => Cigar::Match(len),
                1 => Cigar::Ins(len),
                2 => Cigar::Del(len),
                3 => Cigar::RefSkip(len),
                4 => Cigar::SoftClip(len),
                5 => Cigar::HardClip(len),
                6 => Cigar::Pad(len),
                7 => Cigar::Equal(len),
                8 => Cigar::Diff(len),
                _ => panic!("Unexpected cigar operation"),
            }
        });
        CigarString(if self.is_reverse() {
            cigar_iter.rev().collect()
        } else {
            cigar_iter.collect()
        })
    }

    fn count_sa(&self) -> Result<u16, &'static str> {
        match self.aux(b"SA") {
            Some(Aux::String(v)) => Ok(v.iter().fold(0u16, |acc, &x| acc + (x == b';') as u16)),
            Some(_) => Err("unexpected cigar value"),
            None => Ok(0u16),
        }
    }

    fn first_sa_forward_cigar_string(&self) -> Result<CigarString, &'static str> {
        if let Some(Aux::String(v)) = self.aux(b"SA") {
            let mut chunks = v.split(|&x| x == b',');
            let forward: bool = match chunks.nth(2) {
                Some(v) => v == b"+" as &[u8],
                _ => return Err("unexpected strand oritation"),
            };
            if let Some(s) = chunks.next() {
                if forward {
                    Ok(CigarString::from_bytes(s).unwrap())
                } else {
                    Ok(CigarString(
                        CigarString::from_bytes(s)
                            .unwrap()
                            .iter()
                            .rev()
                            .copied()
                            .collect::<Vec<Cigar>>(),
                    ))
                }
            } else {
                Err("no proper sa cigar")
            }
        } else {
            Err("no supplemental alignment")
        }
    }
}

fn calculate_min_nonoverlap(a: &CigarString, b: &CigarString) -> u32 {
    let mut a_dm: Dimension = Default::default();
    let mut b_dm: Dimension = Default::default();
    let mut mflag: bool = false;
    a.into_iter().for_each(|&x| match x {
        Cigar::SoftClip(v) | Cigar::HardClip(v) => {
            if !mflag {
                a_dm.start += v;
                a_dm.end += v
            }
        }
        Cigar::Match(v) | Cigar::Ins(v) => {
            mflag |= true;
            a_dm.end += v
        }
        _ => (),
    });
    mflag = false;
    b.into_iter().for_each(|&x| match x {
        Cigar::SoftClip(v) | Cigar::HardClip(v) => {
            if !mflag {
                b_dm.start += v;
                b_dm.end += v;
            }
        }
        Cigar::Match(v) | Cigar::Ins(v) => {
            mflag |= true;
            b_dm.end += v;
        }
        _ => (),
    });
    let overlap: u32 = {
        let (num, flag) =
            cmp::min(a_dm.end, b_dm.end).overflowing_sub(cmp::max(a_dm.start, b_dm.start));
        (1u32 - flag as u32) * num + 1
    };
    cmp::min(
        a_dm.end + 1 - a_dm.start - overlap,
        b_dm.end + 1 - b_dm.start - overlap,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pargs = App::new(crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .version(crate_version!())
        .author(crate_authors!(";"))
        .about(crate_description!())
        .args_from_usage(
            "   
            <splitted> -s, --splitted=[file] 'ouput splitted bam path'
            [split_number] -n, --split_number=[number] 'max split number of a split read'
            <discordant> -d, --discordant=[file] 'output discordant bam path'
            [include_dup] -i, --include_dup 'include duplicates'
            [min_nonoverlap] -m, --min_nonoverlap 'minimum non-overlap between split alignments on the query (default=20)'
            <input> 'input bam file'
            ",
        )
        .get_matches();
    let splitnum: u16 = pargs
        .value_of("split_number")
        .unwrap_or("1")
        .parse()
        .unwrap();
    let include_dup: bool = pargs.is_present("include_dup");
    let min_nonoverlap: u32 = pargs
        .value_of("min_nonoverlap")
        .unwrap_or("20")
        .parse()
        .unwrap();
    let mut reader = match BamReader::from_path(pargs.value_of("input").unwrap()) {
        Ok(v) => v,
        Err(e) => panic!("error reading input: {:?}", e),
    };
    let mut writer_s: BamWriter = match pargs.value_of("splitted") {
        Some(v) => {
            if v.split('.').last().unwrap() == "sam" {
                BamWriter::from_path(v, &Header::from_template(reader.header()), Format::SAM)?
            } else {
                BamWriter::from_path(v, &Header::from_template(reader.header()), Format::BAM)?
            }
        }
        None => panic!("output to splitted error"),
    };
    let mut writer_d: BamWriter = match pargs.value_of("discordant") {
        Some(v) => {
            if v.split('.').last().unwrap() == "sam" {
                BamWriter::from_path(v, &Header::from_template(reader.header()), Format::SAM)?
            } else {
                BamWriter::from_path(v, &Header::from_template(reader.header()), Format::BAM)?
            }
        }
        None => panic!("output to discordant error"),
    };
    for (num, i) in reader.records().enumerate() {
        let mut rec = match i {
            Ok(v) => v,
            Err(e) => panic!("line {}: {}", num, e),
        };
        if !include_dup && rec.is_duplicate() {
            continue;
        };
        // write to discordant file
        if rec.flags() & 2318u16 == 0 {
            match writer_d.write(&rec) {
                Ok(_) => (),
                Err(e) => panic!("line {}: {}", num, e),
            };
        };
        // write to splitted file
        let rec_cigar: CigarString = rec.forward_cigar_string();
        match rec.count_sa() {
            Ok(v) => {
                if v > splitnum {
                    continue;
                }
            }
            Err(_) => continue,
        };
        let sa_cigar: CigarString = match rec.first_sa_forward_cigar_string() {
            Ok(v) => v,
            _ => {
                continue;
            }
        };
        if calculate_min_nonoverlap(&rec_cigar, &sa_cigar) < min_nonoverlap {
            continue;
        };
        let appendix = if rec.is_first_in_template() {
            b"_1"
        } else {
            b"_2"
        };
        rec.set_qname(&[rec.qname(), appendix].concat());
        match writer_s.write(&rec) {
            Ok(_) => (),
            Err(e) => panic!("line {}: {}", num, e),
        };
    }
    Ok(())
}
