//! UTC civil calendar conversions shared by library timestamps and recording file names.

use alloc::{format, string::String};
use core::time::Duration;

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_DAY: u64 = 86_400;

const CIVIL_MONTHS_SHIFTED_BEFORE_MARCH: i64 = 9;
const CIVIL_MONTHS_SHIFTED_FROM_MARCH: i64 = -3;
const CIVIL_YEAR_ROLLED_BACK_FOR_JAN_FEB: i64 = 1;

const YEARS_PER_GREGORIAN_ERA: i64 = 400;
const NEGATIVE_ERA_YEAR_ADJUSTMENT: i64 = 399;
const DAYS_PER_GREGORIAN_ERA: i64 = 146_097;
const UNIX_EPOCH_CIVIL_DAY_NUMBER: i64 = 719_468;
const NEGATIVE_ERA_DAY_FLOOR_ADJUSTMENT: i64 = 146_096;

const CIVIL_DAY_OF_YEAR_NUMERATOR: i64 = 153;
const CIVIL_DAY_OF_YEAR_OFFSET: i64 = 2;
const CIVIL_DAY_OF_YEAR_DENOMINATOR: i64 = 5;

const DAYS_PER_COMMON_YEAR: i64 = 365;
const LEAP_YEAR_CYCLE_DIVISOR: i64 = 4;
const CENTURY_YEAR_DIVISOR: i64 = 100;

const DAYS_PER_FOUR_YEAR_BLOCK: u64 = 1_460;
const DAYS_PER_CENTURY_BLOCK: u64 = 36_524;
const DAYS_PER_ERA_REMAINDER: u64 = 146_096;
const YEAR_OF_ERA_LEAP_BOUNDARY: u64 = 4;

const CIVIL_MONTH_INDEX_NUMERATOR: u64 = 5;
const CIVIL_MONTH_INDEX_DENOMINATOR: u64 = 153;
const CIVIL_MONTH_INDEX_MARCH_THRESHOLD: u64 = 10;
const CIVIL_MONTH_FROM_INDEX_BEFORE_MARCH: i64 = -9;
const CIVIL_MONTH_FROM_INDEX_FROM_MARCH: i64 = 3;
const CIVIL_YEAR_INCREMENT_BEFORE_MARCH: i64 = 1;

/// Days since the Unix epoch for a UTC civil date.
pub fn civil_days_since_epoch(year: i64, month: i64, day: i64) -> i64 {
    let year = year
        - if month <= 2 {
            CIVIL_YEAR_ROLLED_BACK_FOR_JAN_FEB
        } else {
            0
        };
    let month = month
        + if month <= 2 {
            CIVIL_MONTHS_SHIFTED_BEFORE_MARCH
        } else {
            CIVIL_MONTHS_SHIFTED_FROM_MARCH
        };
    let era = if year >= 0 {
        year / YEARS_PER_GREGORIAN_ERA
    } else {
        (year - NEGATIVE_ERA_YEAR_ADJUSTMENT) / YEARS_PER_GREGORIAN_ERA
    };
    let year_of_era = year - era * YEARS_PER_GREGORIAN_ERA;
    let day_of_year = (CIVIL_DAY_OF_YEAR_NUMERATOR * month + CIVIL_DAY_OF_YEAR_OFFSET)
        / CIVIL_DAY_OF_YEAR_DENOMINATOR
        + day
        - 1;
    let day_of_era = year_of_era * DAYS_PER_COMMON_YEAR + year_of_era / LEAP_YEAR_CYCLE_DIVISOR
        - year_of_era / CENTURY_YEAR_DIVISOR
        + day_of_year;
    era * DAYS_PER_GREGORIAN_ERA + day_of_era - UNIX_EPOCH_CIVIL_DAY_NUMBER
}

/// Formats wall-clock seconds as `YYYYMMDD_HHMMSS` (UTC) for new recording base names.
pub fn recorder_wall_clock_file_stamp(wall_clock: Duration) -> String {
    let seconds = wall_clock.as_secs();
    let time_of_day = seconds % SECONDS_PER_DAY;
    let hour = time_of_day / SECONDS_PER_HOUR;
    let minute = (time_of_day % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let second = time_of_day % SECONDS_PER_MINUTE;
    let days = seconds / SECONDS_PER_DAY;
    let (year, month, day) = civil_date_from_days_since_epoch(days as i64);
    format!("{year:04}{month:02}{day:02}_{hour:02}{minute:02}{second:02}")
}

/// UTC civil date for days since the Unix epoch (inverse of [`civil_days_since_epoch`]).
pub fn civil_date_from_days_since_epoch(days: i64) -> (i64, i64, i64) {
    let adjusted_days = days + UNIX_EPOCH_CIVIL_DAY_NUMBER;
    let era = if adjusted_days >= 0 {
        adjusted_days
    } else {
        adjusted_days - NEGATIVE_ERA_DAY_FLOOR_ADJUSTMENT
    } / DAYS_PER_GREGORIAN_ERA;
    let days_of_era = (adjusted_days - era * DAYS_PER_GREGORIAN_ERA) as u64;
    let year_of_era = (days_of_era - days_of_era / DAYS_PER_FOUR_YEAR_BLOCK
        + days_of_era / DAYS_PER_CENTURY_BLOCK
        - days_of_era / DAYS_PER_ERA_REMAINDER)
        / DAYS_PER_COMMON_YEAR as u64;
    let mut year = era * YEARS_PER_GREGORIAN_ERA
        + year_of_era as i64
        + (year_of_era == YEAR_OF_ERA_LEAP_BOUNDARY) as i64;
    let day_of_year = days_of_era
        - (DAYS_PER_COMMON_YEAR as u64 * year_of_era
            + year_of_era / LEAP_YEAR_CYCLE_DIVISOR as u64
            - year_of_era / CENTURY_YEAR_DIVISOR as u64);
    let month_index = (CIVIL_MONTH_INDEX_NUMERATOR * day_of_year + CIVIL_DAY_OF_YEAR_OFFSET as u64)
        / CIVIL_MONTH_INDEX_DENOMINATOR;
    let day = day_of_year
        - (CIVIL_DAY_OF_YEAR_NUMERATOR as u64 * month_index + CIVIL_DAY_OF_YEAR_OFFSET as u64)
            / CIVIL_DAY_OF_YEAR_DENOMINATOR as u64
        + 1;
    let month = month_index as i64
        + if month_index < CIVIL_MONTH_INDEX_MARCH_THRESHOLD {
            CIVIL_MONTH_FROM_INDEX_FROM_MARCH
        } else {
            CIVIL_MONTH_FROM_INDEX_BEFORE_MARCH
        };
    year += (month <= 2) as i64 * CIVIL_YEAR_INCREMENT_BEFORE_MARCH;
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
