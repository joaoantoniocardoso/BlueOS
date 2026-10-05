use alloc::format;

use blueos_recorder_paths::civil_days_since_epoch;

const LEGACY_SUFFIX_BODY_LEN: usize = 15;
const RECORDER_PREFIX_MIN_LEN: usize = 15;
const DATE_TIME_SEPARATOR_INDEX: usize = 8;
const TIME_FIELD_START_INDEX: usize = 9;
const RECORDER_PREFIX_TIME_END_INDEX: usize = 15;
const ISO_TIMESTAMP_BODY_LEN: usize = 19;
const ISO_DATE_TIME_SEPARATOR_INDEX: usize = 10;
const DATE_FIELD_LEN: usize = 8;
const TIME_FIELD_LEN: usize = 6;
const YEAR_FIELD_LEN: usize = 4;
const MONTH_FIELD_START: usize = 4;
const MONTH_FIELD_END: usize = 6;
const DAY_FIELD_START: usize = 6;
const DAY_FIELD_END: usize = 8;
const HOUR_FIELD_LEN: usize = 2;
const MINUTE_FIELD_START: usize = 2;
const MINUTE_FIELD_END: usize = 4;
const SECOND_FIELD_START: usize = 4;
const SECOND_FIELD_END: usize = 6;
const MONTH_MIN: i64 = 1;
const MONTH_MAX: i64 = 12;
const DAY_MIN: i64 = 1;
const DAY_MAX: i64 = 31;
const HOUR_MAX: i64 = 24;
const MINUTE_MAX: i64 = 60;
const SECOND_MAX: i64 = 60;
const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 3_600;
const SECONDS_PER_DAY: i64 = 86_400;

struct CivilTimeParts {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
}

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
    if body.len() != LEGACY_SUFFIX_BODY_LEN
        || body.as_bytes().get(DATE_TIME_SEPARATOR_INDEX) != Some(&b'_')
    {
        return None;
    }
    let date = body.get(0..DATE_FIELD_LEN)?;
    let time = body.get(TIME_FIELD_START_INDEX..)?;
    parse_utc_timestamp(date, time)
}

fn parse_recorder_prefix(rest: &str) -> Option<i64> {
    if rest.len() < RECORDER_PREFIX_MIN_LEN
        || rest.as_bytes().get(DATE_TIME_SEPARATOR_INDEX) != Some(&b'_')
    {
        return None;
    }
    let date = rest.get(0..DATE_FIELD_LEN)?;
    let time = rest.get(TIME_FIELD_START_INDEX..RECORDER_PREFIX_TIME_END_INDEX)?;
    parse_utc_timestamp(date, time)
}

fn parse_iso_timestamp_body(body: &str) -> Option<i64> {
    if body.len() != ISO_TIMESTAMP_BODY_LEN {
        return None;
    }
    let separator = body.as_bytes().get(ISO_DATE_TIME_SEPARATOR_INDEX)?;
    if *separator != b'T' && *separator != b't' {
        return None;
    }
    let date = format!(
        "{}{}{}",
        body.get(0..YEAR_FIELD_LEN)?,
        body.get(5..7)?,
        body.get(8..10)?
    );
    let time = format!(
        "{}{}{}",
        body.get(11..13)?,
        body.get(14..16)?,
        body.get(17..19)?
    );
    parse_utc_timestamp(&date, &time)
}

fn parse_utc_timestamp(date: &str, time: &str) -> Option<i64> {
    if date.len() != DATE_FIELD_LEN || time.len() != TIME_FIELD_LEN {
        return None;
    }
    let parts = CivilTimeParts {
        year: parse_digits(date, 0, YEAR_FIELD_LEN)?,
        month: parse_digits(date, MONTH_FIELD_START, MONTH_FIELD_END)?,
        day: parse_digits(date, DAY_FIELD_START, DAY_FIELD_END)?,
        hour: parse_digits(time, 0, HOUR_FIELD_LEN)?,
        minute: parse_digits(time, MINUTE_FIELD_START, MINUTE_FIELD_END)?,
        second: parse_digits(time, SECOND_FIELD_START, SECOND_FIELD_END)?,
    };
    if !valid_utc_parts(&parts) {
        return None;
    }
    Some(unix_timestamp_utc(parts))
}

fn valid_utc_parts(parts: &CivilTimeParts) -> bool {
    (MONTH_MIN..=MONTH_MAX).contains(&parts.month)
        && (DAY_MIN..=DAY_MAX).contains(&parts.day)
        && (0..HOUR_MAX).contains(&parts.hour)
        && (0..MINUTE_MAX).contains(&parts.minute)
        && (0..SECOND_MAX).contains(&parts.second)
}

fn parse_digits(value: &str, start: usize, end: usize) -> Option<i64> {
    value.get(start..end).and_then(|slice| slice.parse().ok())
}

fn unix_timestamp_utc(parts: CivilTimeParts) -> i64 {
    let days = civil_days_since_epoch(parts.year, parts.month, parts.day);
    days * SECONDS_PER_DAY
        + parts.hour * SECONDS_PER_HOUR
        + parts.minute * SECONDS_PER_MINUTE
        + parts.second
}
