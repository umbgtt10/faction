// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::timer::wall_time::WallTime;
use faction_system_tests::traits::wall_clock::WallClock;
use std::time::Duration;

// 2026-11-12T12:30:45Z, in seconds since the epoch: the moment the tests' fake
// wall clocks read.
pub const FAKE_MOMENT: u64 = 1_794_486_645;

pub struct FakeWallClock {
    now: WallTime,
}

impl FakeWallClock {
    pub fn at(seconds_since_epoch: u64) -> Self {
        Self {
            now: WallTime::new(Duration::from_secs(seconds_since_epoch)),
        }
    }
}

impl WallClock for FakeWallClock {
    fn now(&self) -> WallTime {
        self.now
    }
}

#[test]
fn now_of_a_fake_reads_the_day_it_was_set_to() {
    // Arrange
    let clock = FakeWallClock::at(FAKE_MOMENT);

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(reading.to_stamp(), "2026-11-12T12:30:45.000Z");
}
