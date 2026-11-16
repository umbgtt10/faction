// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use chrono::TimeZone;
use chrono::Utc;
use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use faction_system_tests::infra::timer::simulated::simulated_wall_clock::SimulatedWallClock;
use faction_system_tests::traits::wall_clock::WallClock;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn now_reads_the_origin_moved_on_by_the_clock() {
    // Arrange
    let clock = Arc::new(SimulatedClock::at(Duration::from_millis(1_500)));
    let wall_clock = SimulatedWallClock::new(
        clock.clone(),
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
    );
    clock.advance(Duration::from_millis(250));

    // Act
    let reading = wall_clock.now();

    // Assert
    assert_eq!(reading.to_rfc3339(), "2026-01-01T00:00:01.750+00:00");
}
