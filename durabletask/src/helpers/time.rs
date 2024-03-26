use chrono::{Datelike, Timelike};
use prost_types::Timestamp;

pub fn now() -> Timestamp {
    let now = chrono::offset::Local::now();
    Timestamp::date_time(
        now.year() as i64,
        now.month() as u8,
        now.day() as u8,
        now.hour() as u8,
        now.minute() as u8,
        now.second() as u8,
    )
    .unwrap()
}
