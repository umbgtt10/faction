// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::traits::monotonic_clock_tests::FakeMonotonicClock;
use faction_system_tests::traits::monotonic_clock::MonotonicClock;
use faction_system_tests::traits::pause::Pause;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

pub struct FakePause {
    clock: Arc<FakeMonotonicClock>,
    asked: Mutex<Vec<Duration>>,
}

impl FakePause {
    pub fn advancing(clock: Arc<FakeMonotonicClock>) -> Self {
        Self {
            clock,
            asked: Mutex::new(Vec::new()),
        }
    }

    pub fn asked(&self) -> Vec<Duration> {
        self.asked.lock().unwrap().clone()
    }
}

impl Pause for FakePause {
    fn pause(&self, duration: Duration) {
        self.asked.lock().unwrap().push(duration);
        self.clock.advance(duration);
    }
}

#[test]
fn pause_of_a_fake_advances_its_clock_and_records_the_duration() {
    // Arrange
    let clock = Arc::new(FakeMonotonicClock::at(Duration::ZERO));
    let pause = FakePause::advancing(Arc::clone(&clock));

    // Act
    pause.pause(Duration::from_millis(7));

    // Assert
    assert_eq!(clock.now(), Duration::from_millis(7));
    assert_eq!(pause.asked(), vec![Duration::from_millis(7)]);
}
