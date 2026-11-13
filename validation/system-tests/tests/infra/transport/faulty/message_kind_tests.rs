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
fn of_a_membership_acknowledgement_is_membership_acknowledged() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::MembershipAcknowledged { from: 1 });

    // Assert
    assert_eq!(kind, MessageKind::MembershipAcknowledged);
}

#[test]
fn of_a_membership_is_membership() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::Membership {
        from: 1,
        members: vec![1],
        required_count: 1,
    });

    // Assert
    assert_eq!(kind, MessageKind::Membership);
}

#[test]
fn of_a_ping_is_ping() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::Ping { from: 1 });

    // Assert
    assert_eq!(kind, MessageKind::Ping);
}

#[test]
fn of_a_probe_answer_is_probe_answer() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::ProbeAnswer {
        from: 1,
        relay_for: Some(2),
    });

    // Assert
    assert_eq!(kind, MessageKind::ProbeAnswer);
}

#[test]
fn of_a_probe_is_probe() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::Probe {
        from: 1,
        relay_for: None,
    });

    // Assert
    assert_eq!(kind, MessageKind::Probe);
}

#[test]
fn of_a_ready_is_ready() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::Ready { from: 1 });

    // Assert
    assert_eq!(kind, MessageKind::Ready);
}

#[test]
fn of_an_indirect_probe_answer_is_indirect_probe_answer() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::IndirectProbeAnswer { from: 1, target: 2 });

    // Assert
    assert_eq!(kind, MessageKind::IndirectProbeAnswer);
}

#[test]
fn of_an_indirect_probe_is_indirect_probe() {
    // Arrange & Act
    let kind = MessageKind::of(&TransportMessage::IndirectProbe { from: 1, target: 2 });

    // Assert
    assert_eq!(kind, MessageKind::IndirectProbe);
}
