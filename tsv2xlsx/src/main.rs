extern crate xlsxwriter;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use xlsxwriter::{Workbook, XlsxError};

fn main() -> Result<(), Box<dyn Error>> {
    let input: String = env::args().nth(1).expect("Expecting input tsv file.");
    let output: String = env::args().nth(2).unwrap_or(format!("{}.xlsx", input));
    let reader = BufReader::new(File::open(input)?);
    let workbook = Workbook::new(&output);
    let mut sheet = workbook.add_worksheet(None)?;
    let mut rowi = 0;
    reader.lines().try_for_each(|v| {
        let mut coli = 0;
        v?.split("\t").try_for_each(|i| {
            sheet.write_string(rowi, coli, i, None)?;
            coli += 1;
            Ok::<(), XlsxError>(())
        })?;
        rowi += 1;
        Ok::<(), Box<dyn Error>>(())
    })?;
    Ok(())
}
