// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::coordination::Coordination;

#[test]
fn bound_of_a_peer_is_its_own_and_one_past_the_list() {
    // Arrange
    let coordination = Coordination::new(2, vec![1, 3]);

    // Act
    let bounds = [
        coordination.bound(0),
        coordination.bound(1),
        coordination.bound(2),
    ];

    // Assert
    assert_eq!(bounds, [1, 3, 1]);
}

#[test]
fn listed_writes_every_bound_comma_separated() {
    // Arrange
    let coordination = Coordination::new(2, vec![1, 3, 2]);

    // Act
    let listed = coordination.listed();

    // Assert
    assert_eq!(listed, "1,3,2");
}

#[test]
fn suspicion_threshold_is_the_one_given() {
    // Arrange
    let coordination = Coordination::new(2, Vec::new());

    // Act
    let threshold = coordination.suspicion_threshold();

    // Assert
    assert_eq!(threshold, 2);
}
