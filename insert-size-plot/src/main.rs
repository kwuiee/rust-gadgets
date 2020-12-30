#![feature(map_first_last, unsigned_abs)]
// extern crate bam;
extern crate byteorder;
#[macro_use]
extern crate clap;
extern crate flate2;
extern crate plotlib;

use std::fs::File;
use std::io::ErrorKind::{InvalidData, UnexpectedEof};
use std::io::{BufRead, BufReader, Error, Read, Result};

// use bam::bgzip::read::ConsecutiveReader;
use byteorder::{LittleEndian, ReadBytesExt};
use clap::{App, AppSettings};
use flate2::read::MultiGzDecoder;
use plotlib::grid::Grid;
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{LineJoin, LineStyle};
use plotlib::view::{ContinuousView, View};

/// Read is paired, first in pair, properly mapped.
const P_FLAG: u16 = 0x1 + 0x2 + 0x40;
/// Read is secondary or supplementary.
const N_FLAG: u16 = 0x100 + 0x800;

fn opterr() -> std::io::Error {
    Error::new(InvalidData, "Option error.")
}

struct BamReader<T: BufRead> {
    reader: T,
}

impl BamReader<BufReader<MultiGzDecoder<File>>> {
    fn from_path(v: &str) -> Result<Self> {
        let mut file = BufReader::with_capacity(16 * 1024, MultiGzDecoder::new(File::open(v)?));

        // Magic header.
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if magic != [b'B', b'A', b'M', 1] {
            return Err(Error::new(InvalidData, "Wrong BAM magic."));
        };

        // Header text.
        let l_text = file.read_i32::<LittleEndian>()?;
        let mut _text = vec![0u8; l_text as usize];
        file.read_exact(&mut _text)?;

        // Reference and length.
        let n_ref: u32 = file.read_u32::<LittleEndian>()?;
        for _ in 0..n_ref {
            let block_size: usize =
                file.read_u32::<LittleEndian>()? as usize + std::mem::size_of::<u32>();
            let mut _ref_entry = vec![0u8; block_size];
            file.read_exact(&mut _ref_entry)?;
        }

        Ok(Self { reader: file })
    }
}

impl<T: BufRead> BamReader<T> {
    fn read_into(&mut self, record: &mut Record) -> Result<bool> {
        let mut rem_size = match self.reader.read_u32::<LittleEndian>() {
            Ok(value) => value as usize,
            Err(e) => {
                if e.kind() == UnexpectedEof {
                    return Ok(false);
                } else {
                    return Err(e);
                }
            }
        };
        let mut _sink8 = [0u8; 1];
        let mut _sink16 = [0u8; 2];
        let mut _sink32 = [0u8; 4];
        // Ref id.
        record.set_ref_id(self.reader.read_i32::<LittleEndian>()?);
        // Ref position.
        self.reader.read_exact(&mut _sink32)?;
        // Query name length.
        let _l_name = self.reader.read_u8()? as usize;
        // Mapq.
        self.reader.read_exact(&mut _sink8)?;
        // Bin.
        self.reader.read_exact(&mut _sink16)?;
        // Number of operations in CIGAR.
        let _l_cigar = self.reader.read_u16::<LittleEndian>()? as usize;
        // Flag.
        record.set_flag(self.reader.read_u16::<LittleEndian>()?);
        // Sequence length.
        let _l_seq = self.reader.read_u32::<LittleEndian>()? as usize;
        // Mate ref id.
        record.set_mate_ref_id(self.reader.read_i32::<LittleEndian>()?);
        // Mate pos
        self.reader.read_exact(&mut _sink32)?;
        // Template length.
        record.set_tlen(self.reader.read_i32::<LittleEndian>()?);
        // Query name
        self.reader.read_exact(&mut vec![0u8; _l_name])?;
        // Cigar.
        self.reader
            .read_u32_into::<LittleEndian>(&mut vec![0u32; _l_cigar])?;
        // Sequence.
        self.reader.read_exact(&mut vec![0u8; (_l_seq + 1) / 2])?;
        // Quality.
        self.reader.read_exact(&mut vec![0u8; _l_seq])?;
        rem_size -= 32 + _l_name + _l_cigar * 4 + (_l_seq + 1) / 2 + _l_seq;
        // Optinal fields.
        self.reader.read_exact(&mut vec![0u8; rem_size])?;
        Ok(true)
    }
}

#[derive(Default)]
struct Record {
    ref_id: i32,
    mate_ref_id: i32,
    tlen: i32,
    flag: u16,
}

impl Record {
    fn flag(&self) -> &u16 {
        &self.flag
    }

    fn set_flag(&mut self, v: u16) {
        self.flag = v
    }

    fn tlen(&self) -> &i32 {
        &self.tlen
    }

    fn set_tlen(&mut self, v: i32) {
        self.tlen = v
    }

    fn ref_id(&self) -> &i32 {
        &self.ref_id
    }

    fn set_ref_id(&mut self, v: i32) {
        self.ref_id = v
    }

    fn mate_ref_id(&self) -> &i32 {
        &self.mate_ref_id
    }

    fn set_mate_ref_id(&mut self, v: i32) {
        self.mate_ref_id = v
    }
}

fn cli(bam: &str, svg: &str, upper: &usize) -> Result<()> {
    let mut data = vec![0u32; *upper + 1];
    let mut record = Record::default();
    let mut reader = BamReader::from_path(bam)?;
    let mut total = 0;
    while reader.read_into(&mut record)? {
        if record.flag() & P_FLAG != P_FLAG
            || record.flag() & N_FLAG != 0
            || record.ref_id() != record.mate_ref_id()
        {
            continue;
        };
        let tlen = record.tlen().unsigned_abs() as usize;
        if &tlen > upper {
            continue;
        };
        data[tlen] += 1;
        total += 1;
    }
    // Plot line.
    let line = Plot::new(
        data.into_iter()
            .enumerate()
            .map(|(i, j)| (i as f64, (j as f64) / (total as f64)))
            .collect(),
    )
    .line_style(
        LineStyle::new()
            .colour("burlywood")
            .linejoin(LineJoin::Round)
            .width(1.0),
    );
    let mut view = ContinuousView::new()
        .add(line)
        .x_label("插入片段大小(bp)")
        .y_label("比例");
    view.add_grid(Grid::new(4, 4));
    Page::single(&view)
        .save(svg)
        .map_err(|_| Error::new(InvalidData, format!("Failed to write {}", svg)))?;
    Ok(())
}

fn main() -> Result<()> {
    let opts = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .args_from_usage(
            "
            <svg> -o=[FILE] 'Output svg file path.'
            [upper] -m=[NUMBER] 'Maximum insert size to record. Bigger number costs more memory.'
            <bam> 'Input bam file.'
            ",
        )
        .get_matches();
    let bam: &str = opts.value_of("bam").ok_or_else(opterr)?;
    let svg: &str = opts.value_of("svg").ok_or_else(opterr)?;
    let upper: usize = opts
        .value_of("upper")
        .unwrap_or("500")
        .parse()
        .map_err(|_| opterr())?;
    cli(bam, svg, &upper)
}
