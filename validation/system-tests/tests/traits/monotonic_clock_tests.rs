// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::traits::monotonic_clock::MonotonicClock;
use std::sync::Mutex;
use std::time::Duration;

pub struct FakeMonotonicClock {
    now: Mutex<Duration>,
}

impl FakeMonotonicClock {
    pub fn at(now: Duration) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    pub fn advance(&self, by: Duration) {
        *self.now.lock().unwrap() += by;
    }
}

impl MonotonicClock for FakeMonotonicClock {
    fn now(&self) -> Duration {
        *self.now.lock().unwrap()
    }
}

#[test]
fn advance_of_a_fake_moves_its_reading_by_as_much() {
    // Arrange
    let clock = FakeMonotonicClock::at(Duration::from_millis(10));

    // Act
    clock.advance(Duration::from_millis(5));

    // Assert
    assert_eq!(clock.now(), Duration::from_millis(15));
}

#[test]
fn now_of_a_fake_reads_what_it_was_set_to() {
    // Arrange
    let clock = FakeMonotonicClock::at(Duration::from_secs(3));

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(reading, Duration::from_secs(3));
}
