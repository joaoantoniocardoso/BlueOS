//! UTC civil calendar conversions shared by library timestamps and recording file names.

use alloc::{format, string::String};
use core::time::Duration;

/// Days since the Unix epoch for a UTC civil date.
pub fn civil_days_since_epoch(year: i64, month: i64, day: i64) -> i64 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let month = month + if month <= 2 { 9 } else { -3 };
    let era = if year >= 0 {
        year / 400
    } else {
        (year - 399) / 400
    };
    let year_of_era = year - era * 400;
    let day_of_year = (153 * month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Formats wall-clock seconds as `YYYYMMDD_HHMMSS` (UTC) for new recording base names.
pub fn recorder_wall_clock_file_stamp(wall_clock: Duration) -> String {
    let seconds = wall_clock.as_secs();
    let time_of_day = seconds % 86_400;
    let hour = time_of_day / 3_600;
    let minute = (time_of_day % 3_600) / 60;
    let second = time_of_day % 60;
    let days = seconds / 86_400;
    let (year, month, day) = civil_date_from_days_since_epoch(days as i64);
    format!("{year:04}{month:02}{day:02}_{hour:02}{minute:02}{second:02}")
}

/// UTC civil date for days since the Unix epoch (inverse of [`civil_days_since_epoch`]).
pub fn civil_date_from_days_since_epoch(days: i64) -> (i64, i64, i64) {
    let adjusted_days = days + 719_468;
    let era = if adjusted_days >= 0 {
        adjusted_days
    } else {
        adjusted_days - 146_096
    } / 146_097;
    let days_of_era = (adjusted_days - era * 146_097) as u64;
    let year_of_era =
        (days_of_era - days_of_era / 1_460 + days_of_era / 36_524 - days_of_era / 146_096) / 365;
    let mut year = era * 400 + year_of_era as i64 + (year_of_era == 4) as i64;
    let day_of_year = days_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = month_index as i64 + if month_index < 10 { 3 } else { -9 };
    year += (month <= 2) as i64;
    (year, month, day as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_date_round_trips_through_days_since_epoch() {
        for (year, month, day) in [(2024, 1, 2), (2026, 1, 1), (1970, 1, 1)] {
            let days = civil_days_since_epoch(year, month, day);
            assert_eq!(civil_date_from_days_since_epoch(days), (year, month, day));
        }
    }

    #[test]
    fn recorder_prefix_stamp_matches_storage_expectation() {
        let stamp = recorder_wall_clock_file_stamp(Duration::from_secs(1_767_225_600));
        assert_eq!(stamp, "20260101_000000");
    }
}
