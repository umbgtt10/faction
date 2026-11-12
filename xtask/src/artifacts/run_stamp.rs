// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_DAY: u64 = 86_400;

// Days from 0000-03-01 to 1970-01-01, the epoch the civil algorithm below
// counts from. March rather than January so a leap day lands at the end of the
// year and needs no special case.
const DAYS_FROM_CIVIL_EPOCH: i64 = 719_468;
const DAYS_PER_ERA: i64 = 146_097;

// The name of a stage 5 run's log directory, from the moment the run began.
// The moment is handed in rather than read here: the clock is main's to read.
pub struct RunStamp {
    seconds_since_epoch: u64,
}

impl RunStamp {
    pub fn new(seconds_since_epoch: u64) -> Self {
        Self {
            seconds_since_epoch,
        }
    }

    // YYYY-MM-DD_HH-MM-SS, in UTC rather than local time, so the name does not
    // depend on the machine that wrote it and the listing stays in
    // chronological order through a daylight-saving change.
    pub fn to_directory_name(&self) -> String {
        format!("{}_{}", self.date(), self.time_of_day())
    }

    // Howard Hinnant's civil_from_days. Shifts the epoch to 0000-03-01 so
    // February's variable length falls at the end of the year, which is what
    // lets the whole conversion be arithmetic with no lookup table and no
    // leap-year branch.
    fn date(&self) -> String {
        let days_since_unix_epoch = (self.seconds_since_epoch / SECONDS_PER_DAY) as i64;
        let shifted = days_since_unix_epoch + DAYS_FROM_CIVIL_EPOCH;
        let era = shifted.div_euclid(DAYS_PER_ERA);
        let day_of_era = shifted.rem_euclid(DAYS_PER_ERA);

        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;

        let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
        let month = if shifted_month < 10 {
            shifted_month + 3
        } else {
            shifted_month - 9
        };
        let year = year_of_era + era * 400 + i64::from(month <= 2);

        format!("{year:04}-{month:02}-{day:02}")
    }

    fn time_of_day(&self) -> String {
        let seconds_into_day = self.seconds_since_epoch % SECONDS_PER_DAY;
        let hour = seconds_into_day / SECONDS_PER_HOUR;
        let minute = (seconds_into_day % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
        let second = seconds_into_day % SECONDS_PER_MINUTE;

        format!("{hour:02}-{minute:02}-{second:02}")
    }
}
