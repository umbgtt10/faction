// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_DAY: u64 = 86_400;

// Days from 0000-03-01 to 1970-01-01, the epoch the civil algorithm below
// counts from. March rather than January so a leap day lands at the end of the
// year and needs no special case.
const DAYS_FROM_CIVIL_EPOCH: u64 = 719_468;
const DAYS_PER_ERA: u64 = 146_097;

// A moment the wall clock reads, as the time since the Unix epoch, and its
// written forms in UTC: a log line's stamp, a run's folder name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallTime {
    since_epoch: Duration,
}

impl WallTime {
    #[must_use]
    pub fn new(since_epoch: Duration) -> Self {
        Self { since_epoch }
    }

    // The moment `elapsed` later, or the last a duration holds.
    #[must_use]
    pub fn after(self, elapsed: Duration) -> Self {
        Self::new(self.since_epoch.saturating_add(elapsed))
    }

    // The nanoseconds since the epoch, or the most a u64 holds.
    #[must_use]
    pub fn nanoseconds(self) -> u64 {
        u64::try_from(self.since_epoch.as_nanos()).unwrap_or(u64::MAX)
    }

    // YYYY-MM-DD_HH-MM-SS: every field padded, so names sort as text in the
    // order their moments came.
    #[must_use]
    pub fn to_folder_name(self) -> String {
        format!("{}_{}", self.date(), self.time_of_day('-'))
    }

    // RFC 3339 to the millisecond: YYYY-MM-DDTHH:MM:SS.mmmZ.
    #[must_use]
    pub fn to_stamp(self) -> String {
        format!(
            "{}T{}.{:03}Z",
            self.date(),
            self.time_of_day(':'),
            self.since_epoch.subsec_millis()
        )
    }

    // Howard Hinnant's civil_from_days, in unsigned arithmetic: from 0000-03-01
    // no count is negative, and February's variable length falls at the end
    // of the year, so the conversion needs no table and no leap-year branch.
    fn date(self) -> String {
        let shifted = self.since_epoch.as_secs() / SECONDS_PER_DAY + DAYS_FROM_CIVIL_EPOCH;
        let era = shifted / DAYS_PER_ERA;
        let day_of_era = shifted % DAYS_PER_ERA;

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
        let year = year_of_era + era * 400 + u64::from(month <= 2);

        format!("{year:04}-{month:02}-{day:02}")
    }

    // HH, MM and SS, each padded, apart by `separator`.
    fn time_of_day(self, separator: char) -> String {
        let seconds_into_day = self.since_epoch.as_secs() % SECONDS_PER_DAY;
        let hour = seconds_into_day / SECONDS_PER_HOUR;
        let minute = seconds_into_day % SECONDS_PER_HOUR / SECONDS_PER_MINUTE;
        let second = seconds_into_day % SECONDS_PER_MINUTE;
        format!("{hour:02}{separator}{minute:02}{separator}{second:02}")
    }
}
