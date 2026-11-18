// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::engine::fresh_seed::FreshSeed;
use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use faction_system_tests::infra::timer::simulated::simulated_wall_clock::SimulatedWallClock;
use faction_system_tests::infra::timer::wall_time::WallTime;
use std::sync::Arc;
use std::time::Duration;

// A fresh seed read from a wall clock at 2026-01-01, `after` past it.
fn seed_after(after: Duration) -> u64 {
    let clock = Arc::new(SimulatedClock::at(after));
    let origin = WallTime::new(Duration::from_secs(1_767_225_600));
    FreshSeed::new(Arc::new(SimulatedWallClock::new(clock, origin))).seed()
}

#[test]
fn seed_is_the_wall_clock_s_nanoseconds_since_the_epoch() {
    // Arrange & Act
    let seed = seed_after(Duration::ZERO);

    // Assert
    assert_eq!(seed, 1_767_225_600_000_000_000);
}

#[test]
fn seed_of_soaks_a_second_apart_are_a_billion_apart() {
    // Arrange & Act
    let seeds = [
        seed_after(Duration::ZERO),
        seed_after(Duration::from_secs(1)),
    ];

    // Assert
    assert_eq!(seeds[1] - seeds[0], 1_000_000_000);
}
