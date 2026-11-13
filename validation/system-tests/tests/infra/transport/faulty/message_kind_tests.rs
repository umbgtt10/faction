// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::transport::faulty::message_kind::MessageKind;

#[test]
fn default_is_ready() {
    // Arrange & Act
    let kind = MessageKind::default();

    // Assert
    assert_eq!(kind, MessageKind::Ready);
}

#[test]
fn of_a_bootstrapped_message_is_bootstrapped() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::Bootstrapped { from: 1 });

    // Assert
    assert_eq!(kind, MessageKind::Bootstrapped);
}

#[test]
fn of_a_ping_is_ping() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::Ping { from: 1 });

    // Assert
    assert_eq!(kind, MessageKind::Ping);
}

#[test]
fn of_a_ready_is_ready() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::Ready { from: 1 });

    // Assert
    assert_eq!(kind, MessageKind::Ready);
}
