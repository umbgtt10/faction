// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::traits::wall_clock::WallClock;

#[test]
fn now_reads_a_date_after_this_code_was_written() {
    // Arrange
    let clock = SystemWallClock;

    // Act
    let reading = clock.now();

    // Assert
    assert!(reading.timestamp() > 1_762_905_600, "{reading}");
}
