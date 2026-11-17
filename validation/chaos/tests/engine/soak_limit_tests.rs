// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::engine::soak_limit::SoakLimit;
use std::time::Duration;

#[test]
fn reached_by_a_count_of_scenarios_ignores_the_time() {
    // Arrange
    let limit = SoakLimit::Scenarios(10);

    // Act
    let reached = [
        limit.reached(9, Duration::from_secs(9_999)),
        limit.reached(10, Duration::ZERO),
    ];

    // Assert
    assert_eq!(reached, [false, true]);
}

#[test]
fn reached_by_a_duration_ignores_the_count() {
    // Arrange
    let limit = SoakLimit::For(Duration::from_secs(60));

    // Act
    let reached = [
        limit.reached(9_999, Duration::from_secs(59)),
        limit.reached(0, Duration::from_secs(60)),
    ];

    // Assert
    assert_eq!(reached, [false, true]);
}
