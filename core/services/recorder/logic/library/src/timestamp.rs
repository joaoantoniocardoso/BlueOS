use alloc::format;

use blueos_recorder_paths::civil_days_since_epoch;

/// Sort time for a recording, preferring timestamps embedded in the file name.
pub fn created_unix_seconds_from_filename(name: &str, file_time_unix_seconds: i64) -> i64 {
    if !name.is_ascii() {
        return file_time_unix_seconds;
    }
    if let Some(timestamp) = copy_or_snapshot_timestamp(name) {
        return timestamp;
    }
    if let Some(timestamp) = legacy_split_timestamp(name) {
        return timestamp;
    }
    if let Some(timestamp) = recorder_prefix_timestamp(name) {
        return timestamp;
    }
    file_time_unix_seconds
}

fn copy_or_snapshot_timestamp(name: &str) -> Option<i64> {
    let lower = name.to_ascii_lowercase();
    let marker = lower
        .rfind(".copy-")
        .or_else(|| lower.rfind(".snapshot-"))?;
    parse_iso_suffix_timestamp(&lower[marker + 1..])
}

fn legacy_split_timestamp(name: &str) -> Option<i64> {
    let lower = name.to_ascii_lowercase();
    let marker = lower.rfind("_split_")?;
    let suffix = lower.get(marker + "_split_".len()..)?;
    parse_legacy_split_suffix(suffix)
}

fn recorder_prefix_timestamp(name: &str) -> Option<i64> {
    let lower = name.to_ascii_lowercase();
    if !lower.starts_with("recorder_") {
        return None;
    }
    let rest = lower.get("recorder_".len()..)?;
    parse_recorder_prefix(rest)
}

fn parse_iso_suffix_timestamp(suffix: &str) -> Option<i64> {
    let marker = suffix
        .strip_prefix("copy-")
        .or_else(|| suffix.strip_prefix("snapshot-"))?;
    if !marker.ends_with("z.mcap") {
        return None;
    }
    let body = marker.strip_suffix("z.mcap")?;
    parse_iso_timestamp_body(body)
}

fn parse_legacy_split_suffix(suffix: &str) -> Option<i64> {
    if !suffix.ends_with(".mcap") {
        return None;
    }
    let body = suffix.strip_suffix(".mcap")?;
    if body.len() != 15 || body.as_bytes().get(8) != Some(&b'_') {
        return None;
    }
    let date = body.get(0..8)?;
    let time = body.get(9..)?;
    parse_utc_timestamp(date, time)
}

fn parse_recorder_prefix(rest: &str) -> Option<i64> {
    if rest.len() < 15 || rest.as_bytes().get(8) != Some(&b'_') {
        return None;
    }
    let date = rest.get(0..8)?;
    let time = rest.get(9..15)?;
    parse_utc_timestamp(date, time)
}

fn parse_iso_timestamp_body(body: &str) -> Option<i64> {
    if body.len() != 19 {
        return None;
    }
    let separator = body.as_bytes().get(10)?;
    if *separator != b'T' && *separator != b't' {
        return None;
    }
    let date = format!("{}{}{}", body.get(0..4)?, body.get(5..7)?, body.get(8..10)?);
    let time = format!(
        "{}{}{}",
        body.get(11..13)?,
        body.get(14..16)?,
        body.get(17..19)?
    );
    parse_utc_timestamp(&date, &time)
}

fn parse_utc_timestamp(date: &str, time: &str) -> Option<i64> {
    if date.len() != 8 || time.len() != 6 {
        return None;
    }
    let year = parse_digits(date, 0, 4)?;
    let month = parse_digits(date, 4, 6)?;
    let day = parse_digits(date, 6, 8)?;
    let hour = parse_digits(time, 0, 2)?;
    let minute = parse_digits(time, 2, 4)?;
    let second = parse_digits(time, 4, 6)?;
    if !valid_utc_parts(month, day, hour, minute, second) {
        return None;
    }
    Some(unix_timestamp_utc(year, month, day, hour, minute, second))
}

fn valid_utc_parts(month: i64, day: i64, hour: i64, minute: i64, second: i64) -> bool {
    (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && (0..24).contains(&hour)
        && (0..60).contains(&minute)
        && (0..60).contains(&second)
}

fn parse_digits(value: &str, start: usize, end: usize) -> Option<i64> {
    value.get(start..end).and_then(|slice| slice.parse().ok())
}

fn unix_timestamp_utc(year: i64, month: i64, day: i64, hour: i64, minute: i64, second: i64) -> i64 {
    let days = civil_days_since_epoch(year, month, day);
    days * 86_400 + hour * 3600 + minute * 60 + second
}
