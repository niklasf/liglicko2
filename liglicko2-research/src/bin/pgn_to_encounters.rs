use std::{
    io,
    io::{BufRead as _, BufWriter, Write as _},
};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

fn strip_prefix_suffix<'a>(s: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    s.strip_prefix(prefix)?.strip_suffix(suffix)
}

const END_TAG: &str = "\"]";

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock();

    let mut stdout = BufWriter::new(io::stdout().lock());

    let mut white = String::new();
    let mut black = String::new();
    let mut result = String::new();
    let mut utc_date = NaiveDate::default();
    let mut utc_time = NaiveTime::default();
    let mut time_control = String::new();

    let mut line = String::new();
    while stdin.read_line(&mut line)? != 0 {
        if line.ends_with('\n') {
            line.pop();
        }

        if line.is_empty() {
            if !white.is_empty() {
                writeln!(
                    stdout,
                    "{},{},{},{},{}",
                    white,
                    black,
                    result,
                    NaiveDateTime::new(utc_date, utc_time),
                    time_control
                )?;
            }

            white.clear();
            black.clear();
            result.clear();
            utc_date = NaiveDate::default();
            utc_time = NaiveTime::default();
            time_control.clear();
        } else if let Some(v) = strip_prefix_suffix(&line, "[White \"", END_TAG) {
            white.clear();
            white.push_str(v);
            white.make_ascii_lowercase();
        } else if let Some(v) = strip_prefix_suffix(&line, "[Black \"", END_TAG) {
            black.clear();
            black.push_str(v);
            black.make_ascii_lowercase();
        } else if let Some(v) = strip_prefix_suffix(&line, "[Result \"", END_TAG) {
            result.clear();
            result.push_str(v);
        } else if let Some(v) = strip_prefix_suffix(&line, "[UTCDate \"", END_TAG) {
            utc_date = NaiveDate::parse_from_str(v, "%Y.%m.%d").unwrap_or_default();
        } else if let Some(v) = strip_prefix_suffix(&line, "[UTCTime \"", END_TAG) {
            utc_time = v.parse().unwrap_or_default();
        } else if let Some(v) = strip_prefix_suffix(&line, "[TimeControl \"", END_TAG) {
            time_control.clear();
            time_control.push_str(v);
        }

        line.clear();
    }

    Ok(())
}
