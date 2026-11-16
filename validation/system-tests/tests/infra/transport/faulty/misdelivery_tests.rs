// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::transport::faulty::misdelivery::Misdelivery;

#[test]
fn addressee_at_a_hundred_percent_is_another_peer_sent_to_before() {
    // Arrange
    let mut misdelivery = Misdelivery::new(7);
    misdelivery.addressee(2, 0);

    // Act
    let reached = misdelivery.addressee(1, 100);

    // Assert
    assert_eq!(reached, 2);
}

#[test]
fn addressee_at_zero_percent_is_the_peer_addressed() {
    // Arrange
    let mut misdelivery = Misdelivery::new(7);
    misdelivery.addressee(2, 0);

    // Act
    let reached = misdelivery.addressee(1, 0);

    // Assert
    assert_eq!(reached, 1);
}

#[test]
fn addressee_with_no_other_peer_sent_to_is_the_peer_addressed() {
    // Arrange
    let mut misdelivery = Misdelivery::new(7);

    // Act
    let reached = misdelivery.addressee(1, 100);

    // Assert
    assert_eq!(reached, 1);
}
