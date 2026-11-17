// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::live_policy::LivePolicy;
use faction_system_tests::infra::timer::faulty::clock_fault::ClockFault;
use faction_system_tests::infra::timer::faulty::faulty_clock::FaultyClock;
use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use faction_system_tests::traits::monotonic_clock::MonotonicClock;
use std::sync::Arc;
use std::time::Duration;

fn clock(fault: ClockFault) -> (FaultyClock, Arc<SimulatedClock>, LivePolicy<ClockFault>) {
    let inner = Arc::new(SimulatedClock::at(Duration::ZERO));
    let live = LivePolicy::new(fault);
    (FaultyClock::new(live.clone(), inner.clone()), inner, live)
}

#[test]
fn now_after_a_change_of_drift_takes_from_the_reading_before_it() {
    // Arrange
    let (clock, inner, live) = clock(ClockFault {
        drift_ppm: 1_000_000,
        ..ClockFault::default()
    });
    inner.advance(Duration::from_millis(10));
    let fast = clock.now();
    live.set(ClockFault::default());
    inner.advance(Duration::from_millis(10));

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(fast, Duration::from_millis(20));
    assert_eq!(reading, Duration::from_millis(30));
}

#[test]
fn now_of_a_coarse_clock_reads_whole_grains() {
    // Arrange
    let (clock, inner, _) = clock(ClockFault {
        grain: Duration::from_millis(4),
        ..ClockFault::default()
    });
    inner.advance(Duration::from_millis(10));

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(reading, Duration::from_millis(8));
}

#[test]
fn now_of_a_faithful_clock_reads_the_inner_one() {
    // Arrange
    let (clock, inner, _) = clock(ClockFault::default());
    inner.advance(Duration::from_millis(10));

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(reading, Duration::from_millis(10));
}

#[test]
fn now_of_a_fast_clock_runs_ahead_by_its_drift() {
    // Arrange
    let (clock, inner, _) = clock(ClockFault {
        drift_ppm: 1_000_000,
        ..ClockFault::default()
    });
    inner.advance(Duration::from_millis(10));

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(reading, Duration::from_millis(20));
}

#[test]
fn now_of_a_slow_clock_falls_behind_by_its_drift() {
    // Arrange
    let (clock, inner, _) = clock(ClockFault {
        drift_ppm: -500_000,
        ..ClockFault::default()
    });
    inner.advance(Duration::from_millis(10));

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(reading, Duration::from_millis(5));
}

#[test]
fn now_of_a_stopped_clock_stays_where_it_stopped() {
    // Arrange
    let (clock, inner, _) = clock(ClockFault {
        drift_ppm: -2_000_000,
        ..ClockFault::default()
    });
    inner.advance(Duration::from_millis(10));

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(reading, Duration::ZERO);
}

#[test]
fn now_when_its_grain_coarsens_never_runs_back() {
    // Arrange
    let (clock, inner, live) = clock(ClockFault::default());
    inner.advance(Duration::from_millis(10));
    let fine = clock.now();
    live.set(ClockFault {
        grain: Duration::from_millis(4),
        ..ClockFault::default()
    });

    // Act
    let reading = clock.now();

    // Assert
    assert_eq!(fine, Duration::from_millis(10));
    assert_eq!(reading, Duration::from_millis(10));
}
