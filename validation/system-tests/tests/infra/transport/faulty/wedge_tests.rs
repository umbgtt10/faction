// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;
use faction_system_tests::infra::transport::faulty::wedge::Wedge;

fn blocking(unreachable: Vec<u64>) -> FaultPolicy {
    FaultPolicy {
        head_of_line: true,
        unreachable,
        ..FaultPolicy::default()
    }
}

#[test]
fn blocks_a_send_to_an_unreachable_peer_and_every_send_after_it() {
    // Arrange
    let mut wedge = Wedge::default();
    let policy = blocking(vec![2]);

    // Act
    let blocked = [
        wedge.blocks(1, &policy),
        wedge.blocks(2, &policy),
        wedge.blocks(1, &policy),
    ];

    // Assert
    assert_eq!(blocked, [false, true, true]);
}

#[test]
fn blocks_no_more_once_the_peer_is_reachable_again() {
    // Arrange
    let mut wedge = Wedge::default();
    wedge.blocks(2, &blocking(vec![2]));

    // Act
    let blocked = wedge.blocks(1, &blocking(Vec::new()));

    // Assert
    assert!(!blocked);
}

#[test]
fn blocks_nothing_without_head_of_line_blocking() {
    // Arrange
    let mut wedge = Wedge::default();
    let policy = FaultPolicy {
        unreachable: vec![2],
        ..FaultPolicy::default()
    };

    // Act
    let blocked = [wedge.blocks(2, &policy), wedge.blocks(1, &policy)];

    // Assert
    assert_eq!(blocked, [false, false]);
}
