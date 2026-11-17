// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::transport::faulty::first_contact::FirstContact;

#[test]
fn loses_nothing_when_none_are_to_be_lost() {
    // Arrange
    let mut first_contact = FirstContact::default();

    // Act
    let lost = first_contact.loses(1, 0);

    // Assert
    assert!(!lost);
}

#[test]
fn loses_the_first_messages_to_a_peer_and_none_after() {
    // Arrange
    let mut first_contact = FirstContact::default();

    // Act
    let lost: Vec<bool> = (0..4).map(|_| first_contact.loses(1, 2)).collect();

    // Assert
    assert_eq!(lost, vec![true, true, false, false]);
}

#[test]
fn loses_the_first_messages_to_each_peer_apart() {
    // Arrange
    let mut first_contact = FirstContact::default();
    first_contact.loses(1, 1);

    // Act
    let lost = [first_contact.loses(2, 1), first_contact.loses(1, 1)];

    // Assert
    assert_eq!(lost, [true, false]);
}
