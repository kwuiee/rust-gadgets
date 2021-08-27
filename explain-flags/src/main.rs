#[macro_use]
extern crate lazy_static;

use std::collections::BTreeMap;
use std::env;

use colored::*;

// [Explanation source](https://broadinstitute.github.io/picard/explain-flags.html)
lazy_static! {
    static ref EXPLAINS: BTreeMap<u16, &'static str> = {
        let mut m = BTreeMap::new();
        m.insert(0x1, "read paired");
        m.insert(0x2, "read mapped in proper pair");
        m.insert(0x4, "read unmapped");
        m.insert(0x8, "mate unmapped");
        m.insert(0x10, "read reverse strand");
        m.insert(0x20, "mate reverse strand");
        m.insert(0x40, "first in pair");
        m.insert(0x80, "second in pair");
        m.insert(0x100, "not primary alignment");
        m.insert(0x200, "read fails platform/vendor quality checks");
        m.insert(0x400, "read is PCR or optical duplicate");
        m.insert(0x800, "supplementary alignment");
        m
    };
    static ref BITS: u32 = 11;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut s: u16 = env::args().nth(1).expect("Expect a input").parse()?;
    let mut d: u32 = 0;
    while s > 0 {
        match (s % 2, EXPLAINS.get(&2u16.pow(d))) {
            (1, Some(v)) => println!("{} {}", "[\u{2713}]".bold(), v.green().bold()),
            (0, Some(v)) => println!("{} {}", "[\u{2717}]".bold(), v.red().bold()),
            _ => {}
        };
        s /= 2;
        d += 1;
    }
    for i in d..=*BITS {
        if let Some(v) = EXPLAINS.get(&2u16.pow(i)) {
            println!("{} {}", "[\u{2717}]".bold(), v.red().bold());
        };
    }
    Ok(())
}
