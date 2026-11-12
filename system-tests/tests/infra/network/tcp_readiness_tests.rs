// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::fake_time;
use faction_system_tests::infra::network::tcp_readiness::TcpReadiness;
use faction_system_tests::traits::monotonic_clock::MonotonicClock;
use std::net::TcpListener;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::time::Duration;

#[test]
fn wait_on_a_closed_port_panics_once_the_timeout_is_over() {
    // Arrange
    let (time, clock) = fake_time();
    let addr = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let readiness = TcpReadiness::new(time, Duration::from_millis(50));

    // Act
    let outcome = catch_unwind(AssertUnwindSafe(|| readiness.wait(addr)));

    // Assert
    assert!(outcome.is_err());
    assert_eq!(clock.now(), Duration::from_millis(50));
}

#[test]
fn wait_on_a_listening_port_returns_at_once() {
    // Arrange
    let (time, clock) = fake_time();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let readiness = TcpReadiness::new(time, Duration::from_secs(1));

    // Act
    readiness.wait(listener.local_addr().unwrap());

    // Assert
    assert_eq!(clock.now(), Duration::ZERO);
}
