use chrono::{Datelike, Local, TimeZone, Timelike};

pub struct DateTimeUtils;

impl DateTimeUtils {
    pub fn format_time_date(date: i64) -> String {
        let datetime = Local.timestamp_opt(date, 0).unwrap();
        format!(
            "{:02}:{:02} {:02}/{:02}/{:04}",
            datetime.hour(),
            datetime.minute(),
            datetime.day(),
            datetime.month(),
            datetime.year()
        )
    }

    pub fn format_duration(seconds: u64) -> String {
        let h = seconds / 3600;
        let m = (seconds % 3600) / 60;
        let s = seconds % 60;

        let mut res = if h > 0 {
            format!("{:02}:{:02}:{:02}", h, m, s)
        } else if m > 0 {
            format!("{:02}:{:02}", m, s)
        } else {
            format!("{s}")
        };

        while res.starts_with("0") {
            res.remove(0);
        }

        res
    }

    pub fn format_date(date: i64) -> String {
        let datetime = Local.timestamp_opt(date, 0).unwrap();
        format!(
            "{:02}/{:02}/{:04}",
            datetime.day(),
            datetime.month(),
            datetime.year()
        )
    }
}
