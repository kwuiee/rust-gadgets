extern crate chrono;

use std::error::Error;

use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::Duration;
use chrono::{Date, Datelike};

fn main() -> Result<(), Box<dyn Error>> {
    let date: Date<Local> = Local::today();
    let padding = date.weekday().num_days_from_monday();
    let start = {
        let tmp = date - Duration::days(padding as i64);
        if tmp.year() != date.year() {
            Date::<Local>::from_utc(NaiveDate::from_ymd(date.year(), 1, 1), *date.offset())
        } else {
            tmp
        }
    };
    let end = {
        let tmp = date + Duration::days((6 - padding) as i64);
        if tmp.year() != date.year() {
            tmp - Duration::days(tmp.day() as i64)
        } else {
            tmp
        }
    };
    println!(
        "Week {}: {}-{}",
        date.format("%W").to_string().parse::<u32>()? + 1,
        start.format("%Y/%m/%d"),
        end.format("%Y/%m/%d")
    );
    Ok(())
}
