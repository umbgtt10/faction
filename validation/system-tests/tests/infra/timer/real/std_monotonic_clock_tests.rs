// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::timer::real::std_monotonic_clock::StdMonotonicClock;
use faction_system_tests::traits::monotonic_clock::MonotonicClock;

#[test]
fn now_never_reads_earlier_than_it_read_before() {
    // Arrange
    let clock = StdMonotonicClock::new();
    let first = clock.now();

    // Act
    let second = clock.now();

    // Assert
    assert!(second >= first, "{second:?} after {first:?}");
}
