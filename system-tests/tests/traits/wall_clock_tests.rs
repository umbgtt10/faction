// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use faction_system_tests::traits::wall_clock::WallClock;

pub struct FakeWallClock {
    now: DateTime<Utc>,
}

impl FakeWallClock {
    pub fn at(year: i32, month: u32, day: u32) -> Self {
        Self {
            now: Utc.with_ymd_and_hms(year, month, day, 12, 30, 45).unwrap(),
        }
    }
}

impl WallClock for FakeWallClock {
    fn now(&self) -> DateTime<Utc> {
        self.now
    }
}

#[test]
fn now_of_a_fake_reads_the_day_it_was_set_to() {
    // Arrange
    let clock = FakeWallClock::at(2026, 11, 12);

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(reading.to_rfc3339(), "2026-11-12T12:30:45+00:00");
}
