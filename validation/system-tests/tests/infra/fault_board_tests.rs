// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::fault_board::FaultBoard;
use faction_system_tests::infra::timer::faulty::clock_fault::ClockFault;
use faction_system_tests::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;

#[test]
fn node_hands_every_caller_the_same_faults() {
    // Arrange
    let board = FaultBoard::default();
    let dead = TimerFaultPolicy {
        dead: true,
        ..TimerFaultPolicy::default()
    };
    board.node(1).timer.set(dead);

    // Act
    let faults = board.node(1);

    // Assert
    assert_eq!(faults.timer.current(), dead);
}

#[test]
fn node_keeps_each_peers_faults_apart() {
    // Arrange
    let board = FaultBoard::default();
    board.node(1).transport.set(FaultPolicy {
        loss: 100,
        ..FaultPolicy::default()
    });

    // Act
    let other = board.node(2);

    // Assert
    assert_eq!(other.transport.current(), FaultPolicy::default());
}

#[test]
fn node_of_a_peer_never_set_is_faithful() {
    // Arrange
    let board = FaultBoard::default();

    // Act
    let faults = board.node(3);

    // Assert
    assert_eq!(faults.transport.current(), FaultPolicy::default());
    assert_eq!(faults.timer.current(), TimerFaultPolicy::default());
    assert_eq!(faults.clock.current(), ClockFault::default());
}
