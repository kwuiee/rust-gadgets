#![feature(map_first_last, unsigned_abs)]
extern crate bam;
#[macro_use]
extern crate clap;
extern crate sled;

use std::collections::BTreeMap;
use std::convert::From;
use std::convert::TryInto;
use std::error::Error;
use std::iter::Iterator;

use bam::BamReader;
use clap::{App, AppSettings};
use sled::Db as Sled;
use sled::{Config, IVec};

struct Bookkeeper {
    inner: Sled,
    cache: BTreeMap<u32, u32>,
    capacity: usize,
}

impl Bookkeeper {
    fn new() -> Result<Self, sled::Error> {
        let config = Config::new().temporary(true);
        Ok(Self {
            inner: config.open()?,
            cache: BTreeMap::new(),
            capacity: 200,
        })
    }

    fn value_mut(&mut self, v: &u32) -> Result<&mut u32, Box<dyn Error>> {
        if self.cache.contains_key(&v) {
            return Ok(self.cache.get_mut(&v).unwrap());
        };
        let bv = u32::to_le_bytes(*v);
        match self.inner.remove(&bv)? {
            Some(old) => {
                if self.cache.len() > self.capacity {
                    let (key, value) = self.cache.pop_first().unwrap();
                    self.inner
                        .insert(&u32::to_le_bytes(key), &u32::to_le_bytes(value))?;
                };
                self.cache
                    .insert(*v, u32::from_le_bytes(<&[u8]>::from(&old).try_into()?));
                Ok(self.cache.get_mut(&v).unwrap())
            }
            None => {
                if self.cache.len() > self.capacity {
                    let (key, value) = self.cache.pop_first().unwrap();
                    self.inner
                        .insert(&u32::to_le_bytes(key), &u32::to_le_bytes(value))?;
                };
                self.cache.insert(*v, 0);
                Ok(self.cache.get_mut(&v).unwrap())
            }
        }
    }

    fn write_csv(&mut self, output: &str) -> Result<(), Box<dyn Error>> {
        let mut output = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_path(output)?;
        output.write_record(&[b"length", b"number"])?;
        self.inner.iter().try_for_each(|each| {
            let (key, value): (IVec, IVec) = each?;
            output.write_record(&[
                u32::from_le_bytes(key.as_ref().try_into()?)
                    .to_string()
                    .as_bytes(),
                u32::from_le_bytes(value.as_ref().try_into()?)
                    .to_string()
                    .as_bytes(),
            ])?;
            Ok::<(), Box<dyn Error>>(())
        })?;
        while let Some((key, value)) = self.cache.pop_first() {
            output.write_record(&[key.to_string().as_bytes(), value.to_string().as_bytes()])?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .args_from_usage(
            "
            <output> -o, --output=[file] 'Output file path.'
            <bam> 'Input bam file.'
            ",
        )
        .get_matches();
    let bam = opts.value_of("bam").unwrap();
    let output = opts.value_of("output").unwrap();
    let mut bk = Bookkeeper::new()?;
    for i in BamReader::from_path(bam, 0)? {
        bk.value_mut(&i?.template_len().unsigned_abs())
            .map(|v| *v += 1)?;
    }
    bk.write_csv(output)?;
    Ok(())
}
