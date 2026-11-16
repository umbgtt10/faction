// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use faction_system_tests::traits::monotonic_clock::MonotonicClock;
use faction_system_tests::traits::pause::Pause;
use std::time::Duration;

#[test]
fn advance_moves_the_reading_by_as_much() {
    // Arrange
    let clock = SimulatedClock::at(Duration::from_millis(10));

    // Act
    clock.advance(Duration::from_millis(5));

    // Assert
    assert_eq!(clock.now(), Duration::from_millis(15));
}

#[test]
fn now_reads_what_the_clock_was_set_to() {
    // Arrange
    let clock = SimulatedClock::at(Duration::from_secs(3));

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(reading, Duration::from_secs(3));
}

#[test]
fn pause_moves_the_reading_by_the_duration_paused() {
    // Arrange
    let clock = SimulatedClock::at(Duration::from_millis(10));

    // Act
    clock.pause(Duration::from_millis(7));

    // Assert
    assert_eq!(clock.now(), Duration::from_millis(17));
}
