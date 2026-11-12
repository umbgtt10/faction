// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::timer_delay::TimerDelay;
use std::time::Duration;

#[test]
fn duration_of_generous_is_two_hundred_milliseconds() {
    // Arrange & Act
    let duration = TimerDelay::Generous.duration();

    // Assert
    assert_eq!(duration, Duration::from_millis(200));
}

#[test]
fn duration_of_minimal_is_ten_milliseconds() {
    // Arrange & Act
    let duration = TimerDelay::Minimal.duration();

    // Assert
    assert_eq!(duration, Duration::from_millis(10));
}

#[test]
fn duration_of_moderate_is_fifty_milliseconds() {
    // Arrange & Act
    let duration = TimerDelay::Moderate.duration();

    // Assert
    assert_eq!(duration, Duration::from_millis(50));
}
