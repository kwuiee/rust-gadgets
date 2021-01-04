extern crate bio;
#[macro_use]
extern crate clap;
extern crate flate2;

use std::fs::File;
use std::io::ErrorKind::InvalidData;
use std::io::{Error, ErrorKind, Read, Result};
use std::iter::Iterator;

use bio::io::fastq::Reader as FastqReader;
use bio::io::fastq::Record as FastqRecord;
use bio::io::fastq::Records as FastqRecords;
use clap::{App, AppSettings};
use flate2::read::MultiGzDecoder;

use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{LineJoin, LineStyle};
use plotlib::view::ContinuousView;

const GZ_MAGIC: [u8; 3] = [0x1f, 0x8b, 0x08];

fn opterr() -> Error {
    Error::new(InvalidData, "Option error.")
}

#[allow(unused_must_use)]
fn is_gzipped(v: &str) -> Result<bool> {
    let mut magic = [0u8; 3];
    File::open(v).map(|mut v| v.read(&mut magic))?;
    Ok(magic == GZ_MAGIC && (v.ends_with(".gz") || v.ends_with(".gzip")))
}

struct Reader {
    records1: FastqRecords<Box<dyn Read>>,
    records2: FastqRecords<Box<dyn Read>>,
}

impl Reader {
    fn from_path(read1: &str, read2: &str) -> Result<Self> {
        // Is read1 gzipped
        let reader1: FastqReader<Box<dyn Read>> = if is_gzipped(read1)? {
            FastqReader::new(Box::new(MultiGzDecoder::new(File::open(read1)?)))
        } else {
            FastqReader::new(Box::new(File::open(read1)?))
        };
        // Is read2 gzipped
        let reader2: FastqReader<Box<dyn Read>> = if is_gzipped(read2)? {
            FastqReader::new(Box::new(MultiGzDecoder::new(File::open(read2)?)))
        } else {
            FastqReader::new(Box::new(File::open(read2)?))
        };
        let records1 = reader1.records();
        let records2 = reader2.records();
        Ok(Self { records1, records2 })
    }
}

impl Iterator for Reader {
    type Item = Result<(FastqRecord, FastqRecord)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.records1.next().zip(self.records2.next()) {
            Some((Ok(v1), Ok(v2))) => Some(Ok((v1, v2))),
            Some((Err(v), _)) | Some((_, Err(v))) => Some(Err(v)),
            _ => None,
        }
    }
}

trait VecExt<T> {
    fn inc(&mut self, index: usize);
    fn set(&mut self, index: usize, value: T);
}

impl VecExt<f64> for Vec<f64> {
    fn inc(&mut self, index: usize) {
        if self.len() <= index {
            self.resize(index + 1, 0 as f64)
        }
        if let Some(v) = self.get_mut(index) {
            *v += 1f64;
        }
    }

    fn set(&mut self, index: usize, value: f64) {
        if self.len() <= index {
            self.resize(index + 1, 0 as f64)
        }
        self[index] = value;
    }
}

impl VecExt<usize> for Vec<usize> {
    fn inc(&mut self, index: usize) {
        if self.len() < index {
            self.resize(index + 1, 0usize)
        }
        if let Some(v) = self.get_mut(index) {
            *v += 1usize;
        };
    }

    fn set(&mut self, index: usize, value: usize) {
        if self.len() < index {
            self.resize(index + 1, 0usize)
        }
        self[index] = value;
    }
}

enum BaseType {
    A,
    T,
    C,
    G,
    N,
}

impl BaseType {
    fn guess(v: u8) -> Self {
        match v {
            65 | 97 => Self::A,
            84 | 116 => Self::T,
            67 | 99 => Self::C,
            71 | 103 => Self::G,
            78 | 110 => Self::N,
            _ => {
                // FIXME: maybe panic or return error.
                Self::N
            }
        }
    }
}

fn qual2err(v: f64) -> f64 {
    // Convert Phred+33 to error rate.
    10f64.powf((32f64 - v) / 10f64)
}

fn round_max(mut v: f64) -> f64 {
    let mut digits = 0i32;
    if v >= 10f64 {
        while v >= 10f64 {
            v /= 10f64;
            digits += 1;
        }
    } else if v < 1f64 {
        while v < 1f64 {
            v *= 10f64;
            digits -= 1;
        }
    }
    (v.ceil() + 0.1f64) * 10f64.powi(digits)
}

struct Sum {
    base_a1: Vec<usize>,
    base_t1: Vec<usize>,
    base_c1: Vec<usize>,
    base_g1: Vec<usize>,
    base_n1: Vec<usize>,
    quality1: Vec<f64>,
    error1: Vec<f64>,
    base_a2: Vec<usize>,
    base_t2: Vec<usize>,
    base_c2: Vec<usize>,
    base_g2: Vec<usize>,
    base_n2: Vec<usize>,
    quality2: Vec<f64>,
    error2: Vec<f64>,
}

impl Sum {
    /// Create with capacity.
    ///
    /// Capacity should equal to size of read1 + read2.
    fn with_capacity(v: usize) -> Self {
        Self {
            base_a1: Vec::with_capacity(v),
            base_t1: Vec::with_capacity(v),
            base_c1: Vec::with_capacity(v),
            base_g1: Vec::with_capacity(v),
            base_n1: Vec::with_capacity(v),
            quality1: Vec::with_capacity(v),
            error1: Vec::with_capacity(v),
            base_a2: Vec::with_capacity(v),
            base_t2: Vec::with_capacity(v),
            base_c2: Vec::with_capacity(v),
            base_g2: Vec::with_capacity(v),
            base_n2: Vec::with_capacity(v),
            quality2: Vec::with_capacity(v),
            error2: Vec::with_capacity(v),
        }
    }

    fn new() -> Self {
        Self::with_capacity(150)
    }

    fn depth1(&self, index: usize) -> usize {
        self.base_a1.get(index).unwrap_or(&0usize)
            + self.base_t1.get(index).unwrap_or(&0usize)
            + self.base_c1.get(index).unwrap_or(&0usize)
            + self.base_g1.get(index).unwrap_or(&0usize)
            + self.base_n1.get(index).unwrap_or(&0usize)
    }

    fn len1(&self) -> usize {
        self.base_a1
            .len()
            .max(self.base_t1.len())
            .max(self.base_c1.len())
            .max(self.base_g1.len())
            .max(self.base_n1.len())
    }

    fn depth2(&self, index: usize) -> usize {
        self.base_a2.get(index).unwrap_or(&0usize)
            + self.base_t2.get(index).unwrap_or(&0usize)
            + self.base_c2.get(index).unwrap_or(&0usize)
            + self.base_g2.get(index).unwrap_or(&0usize)
            + self.base_n2.get(index).unwrap_or(&0usize)
    }

    fn len2(&self) -> usize {
        self.base_a2
            .len()
            .max(self.base_t2.len())
            .max(self.base_c2.len())
            .max(self.base_g2.len())
            .max(self.base_n2.len())
    }

    fn capture_add_pair(&mut self, record: (FastqRecord, FastqRecord)) {
        // Iterator over first record.
        for (index, (base, qual)) in record
            .0
            .seq()
            .iter()
            .zip(record.0.qual().iter())
            .enumerate()
        {
            let depth1 = self.depth1(index) as f64;
            let new_err = (self.error1.get(index).unwrap_or(&0f64) * depth1
                + qual2err(*qual as f64))
                / (depth1 + 1f64);
            self.error1.set(index, new_err);
            let new_qual = (self.quality1.get(index).unwrap_or(&0f64) * depth1 + *qual as f64)
                / (depth1 + 1f64);
            self.quality1.set(index, new_qual);
            match BaseType::guess(*base) {
                BaseType::A => self.base_a1.inc(index),
                BaseType::T => self.base_t1.inc(index),
                BaseType::C => self.base_c1.inc(index),
                BaseType::G => self.base_g1.inc(index),
                _ => self.base_n1.inc(index),
            };
        }
        // Iterator over second record.
        for (index, (base, qual)) in record
            .1
            .seq()
            .iter()
            .zip(record.1.qual().iter())
            .enumerate()
        {
            let depth2 = self.depth2(index) as f64;
            let new_err = (self.error2.get(index).unwrap_or(&0f64) * depth2
                + qual2err(*qual as f64))
                / (depth2 + 1f64);
            self.error2.set(index, new_err);
            let new_qual = (self.quality2.get(index).unwrap_or(&0f64) * depth2 + *qual as f64)
                / (depth2 + 1f64);
            self.quality2.set(index, new_qual);
            match BaseType::guess(*base) {
                BaseType::A => self.base_a2.inc(index),
                BaseType::T => self.base_t2.inc(index),
                BaseType::C => self.base_c2.inc(index),
                BaseType::G => self.base_g2.inc(index),
                _ => self.base_n2.inc(index),
            };
        }
    }

    /// Plot content, quality, error and save with prefix prefix, namely, {prefix}_gc.svg,
    /// {prefix}_qual.svg and {prefix}_err.svg.
    fn plot(&self, prefix: &str) -> Result<()> {
        // Plot GC.
        let len1 = self.len1();
        let mut a1 = Vec::<(f64, f64)>::with_capacity(len1);
        let mut t1 = Vec::<(f64, f64)>::with_capacity(len1);
        let mut c1 = Vec::<(f64, f64)>::with_capacity(len1);
        let mut g1 = Vec::<(f64, f64)>::with_capacity(len1);
        let mut n1 = Vec::<(f64, f64)>::with_capacity(len1);
        let mut q1 = Vec::<(f64, f64)>::with_capacity(len1);
        let mut e1 = Vec::<(f64, f64)>::with_capacity(len1);
        let mut b_max = 0f64;
        let mut q_max = 0f64;
        let mut e_max = 0f64;
        (0..len1).for_each(|v| {
            let mut y =
                (*self.base_a1.get(v).unwrap_or(&0usize) as f64) / (self.depth1(v) as f64) * 100f64;
            b_max = f64::max(b_max, y);
            a1.push((v as f64, y));
            y = (*self.base_t1.get(v).unwrap_or(&0usize) as f64) / (self.depth1(v) as f64) * 100f64;
            b_max = f64::max(b_max, y);
            t1.push((v as f64, y));
            y = (*self.base_c1.get(v).unwrap_or(&0usize) as f64) / (self.depth1(v) as f64) * 100f64;
            b_max = f64::max(b_max, y);
            c1.push((v as f64, y));
            y = (*self.base_g1.get(v).unwrap_or(&0usize) as f64) / (self.depth1(v) as f64) * 100f64;
            b_max = f64::max(b_max, y);
            g1.push((v as f64, y));
            y = (*self.base_n1.get(v).unwrap_or(&0usize) as f64) / (self.depth1(v) as f64) * 100f64;
            b_max = f64::max(b_max, y);
            n1.push((v as f64, y));
            y = *self.quality1.get(v).unwrap_or(&0f64);
            q_max = f64::max(q_max, y);
            q1.push((v as f64, y));
            y = *self.error1.get(v).unwrap_or(&0f64);
            e_max = f64::max(e_max, y);
            e1.push((v as f64, e_max));
        });
        let a1 = Plot::new(a1)
            .line_style(
                LineStyle::new()
                    .colour("#5150ad")
                    .linejoin(LineJoin::Round)
                    .width(1.0),
            )
            .legend("A".to_string());
        let t1 = Plot::new(t1)
            .line_style(
                LineStyle::new()
                    .colour("#50ada9")
                    .linejoin(LineJoin::Round)
                    .width(1.0),
            )
            .legend("T".to_string());
        let c1 = Plot::new(c1)
            .line_style(
                LineStyle::new()
                    .colour("#BEBEBE")
                    .linejoin(LineJoin::Round)
                    .width(1.0),
            )
            .legend("C".to_string());
        let g1 = Plot::new(g1)
            .line_style(
                LineStyle::new()
                    .colour("#FA8072")
                    .linejoin(LineJoin::Round)
                    .width(1.0),
            )
            .legend("G".to_string());
        let n1 = Plot::new(n1)
            .line_style(
                LineStyle::new()
                    .colour("#FF0000")
                    .linejoin(LineJoin::Round)
                    .width(1.0),
            )
            .legend("N".to_string());

        let len2 = self.len2();
        let mut a2 = Vec::<(f64, f64)>::with_capacity(len2);
        let mut t2 = Vec::<(f64, f64)>::with_capacity(len2);
        let mut c2 = Vec::<(f64, f64)>::with_capacity(len2);
        let mut g2 = Vec::<(f64, f64)>::with_capacity(len2);
        let mut n2 = Vec::<(f64, f64)>::with_capacity(len2);
        let mut q2 = Vec::<(f64, f64)>::with_capacity(len1);
        let mut e2 = Vec::<(f64, f64)>::with_capacity(len1);
        (0..len2).for_each(|v| {
            let mut y =
                (*self.base_a2.get(v).unwrap_or(&0usize) as f64) / (self.depth2(v) as f64) * 100f64;
            a2.push(((v + len1) as f64, y));
            b_max = f64::max(b_max, y);
            y = (*self.base_t2.get(v).unwrap_or(&0usize) as f64) / (self.depth2(v) as f64) * 100f64;
            b_max = f64::max(b_max, y);
            t2.push(((v + len1) as f64, y));
            y = (*self.base_c2.get(v).unwrap_or(&0usize) as f64) / (self.depth2(v) as f64) * 100f64;
            b_max = f64::max(b_max, y);
            c2.push(((v + len1) as f64, y));
            y = (*self.base_g2.get(v).unwrap_or(&0usize) as f64) / (self.depth2(v) as f64) * 100f64;
            b_max = f64::max(b_max, y);
            g2.push(((v + len1) as f64, y));
            y = (*self.base_n2.get(v).unwrap_or(&0usize) as f64) / (self.depth2(v) as f64) * 100f64;
            b_max = f64::max(b_max, y);
            n2.push(((v + len1) as f64, y));
            y = *self.quality2.get(v).unwrap_or(&0f64);
            q_max = f64::max(q_max, y);
            q2.push(((v + len1) as f64, y));
            y = *self.error2.get(v).unwrap_or(&0f64);
            e_max = f64::max(e_max, y);
            e2.push(((v + len1) as f64, y));
        });
        let a2 = Plot::new(a2).line_style(
            LineStyle::new()
                .colour("#5150ad")
                .linejoin(LineJoin::Round)
                .width(1.0),
        );
        let t2 = Plot::new(t2).line_style(
            LineStyle::new()
                .colour("#50ada9")
                .linejoin(LineJoin::Round)
                .width(1.0),
        );
        let c2 = Plot::new(c2).line_style(
            LineStyle::new()
                .colour("#BEBEBE")
                .linejoin(LineJoin::Round)
                .width(1.0),
        );
        let g2 = Plot::new(g2).line_style(
            LineStyle::new()
                .colour("#FA8072")
                .linejoin(LineJoin::Round)
                .width(1.0),
        );
        let n2 = Plot::new(n2).line_style(
            LineStyle::new()
                .colour("#FF0000")
                .linejoin(LineJoin::Round)
                .width(1.0),
        );
        b_max = f64::max(round_max(b_max), 51f64);
        let view = ContinuousView::new()
            .x_range(0f64, (len1 + len2 + 1) as f64)
            .x_label("测序片段位置")
            .y_label("比例(%)")
            .y_range(0f64, b_max)
            .add(a1)
            .add(t1)
            .add(c1)
            .add(g1)
            .add(n1)
            .add(a2)
            .add(t2)
            .add(c2)
            .add(g2)
            .add(n2)
            .add(
                Plot::new(vec![(len1 as f64, 0f64), (len1 as f64, b_max)])
                    .line_style(LineStyle::new().colour("#FFE4C4").width(1.0)),
            );
        Page::single(&view)
            .save(format!("{}gc.svg", prefix))
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        // Plot qual.
        q_max = f64::max(round_max(q_max), 41f64);
        let view = ContinuousView::new()
            .x_range(0f64, (len1 + len2 + 1) as f64)
            .x_label("测序片段位置")
            .y_label("质量值")
            .y_range(0f64, q_max)
            .add(
                Plot::new(q1).line_style(
                    LineStyle::new()
                        .colour("#FF0000")
                        .linejoin(LineJoin::Round)
                        .width(1.0),
                ),
            )
            .add(
                Plot::new(q2).line_style(
                    LineStyle::new()
                        .colour("#FF0000")
                        .linejoin(LineJoin::Round)
                        .width(1.0),
                ),
            )
            .add(
                Plot::new(vec![(len1 as f64, 0f64), (len1 as f64, q_max)])
                    .line_style(LineStyle::new().colour("#FFE4C4").width(1.0)),
            );
        Page::single(&view)
            .save(format!("{}qual.svg", prefix))
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        // Plot error.
        e_max = f64::max(round_max(e_max), 0.01f64);
        let view = ContinuousView::new()
            .x_range(0f64, (len1 + len2 + 1) as f64)
            .x_label("测序片段位置")
            .y_label("错误率(%)")
            .y_range(0f64, e_max)
            .add(
                Plot::new(e1).line_style(
                    LineStyle::new()
                        .colour("#FF0000")
                        .linejoin(LineJoin::Round)
                        .width(1.0),
                ),
            )
            .add(
                Plot::new(e2).line_style(
                    LineStyle::new()
                        .colour("#FF0000")
                        .linejoin(LineJoin::Round)
                        .width(1.0),
                ),
            )
            .add(
                Plot::new(vec![(len1 as f64, 0f64), (len1 as f64, e_max)])
                    .line_style(LineStyle::new().colour("#FFE4C4").width(1.0)),
            );
        Page::single(&view)
            .save(format!("{}err.svg", prefix))
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let opts = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .args_from_usage(
            "
            <prefix> -o, --prefix=[FILE] 'Output prefix.'
            <read1> -1, --read1=[FILE] 'Fastq of read1.'
            <read2> -2, --read2=[FILE] 'Fastq of read2.'
            ",
        )
        .get_matches();
    let prefix: &str = opts.value_of("prefix").ok_or_else(opterr)?;
    let mut reader: Reader = match (opts.value_of("read1"), opts.value_of("read2")) {
        (Some(v1), Some(v2)) => Reader::from_path(v1, v2)?,
        _ => return Err(opterr()),
    };
    let mut sum = Sum::new();

    reader.try_for_each(|pair| {
        sum.capture_add_pair(pair?);
        Ok::<(), Error>(())
    })?;

    sum.plot(prefix)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_round_max() {
        assert_eq!(round_max(0.0035f64), 0.00401f64);
        assert_eq!(round_max(55f64), 60.1f64);
    }
}
