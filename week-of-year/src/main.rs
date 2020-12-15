extern crate chrono;

use std::error::Error;
use std::string::ToString;

use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::{Date, Datelike, Weekday};

fn main() -> Result<(), Box<dyn Error>> {
    let date: Date<Local> = Local::today();
    let padded: bool = matches!(
        Date::<Local>::from_utc(NaiveDate::from_ymd(date.year(), 1, 1), *date.offset()).weekday(),
        Weekday::Mon
    );
    let woy = {
        let weeks = date.format("%j").to_string().parse::<f32>()? / 7.0;
        if weeks.fract() > 0.0 {
            Ok::<u32, Box<dyn Error>>(weeks.ceil() as u32)
        } else if !padded {
            Ok(weeks.trunc() as u32 + 1)
        } else {
            Ok(weeks.trunc() as u32)
        }
    }?;
    println!("Week of this year: {}", woy);
    Ok(())
}
