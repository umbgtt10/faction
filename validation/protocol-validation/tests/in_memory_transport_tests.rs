// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol_validation::in_memory_transport::InMemoryTransport;

#[test]
fn connect_lets_a_late_peer_reach_a_member_and_be_reached() {
    // Arrange
    let mut member = InMemoryTransport::new_mesh(&[0]).remove(0);
    let mut late = InMemoryTransport::new_mesh(&[9]).remove(0);

    // Act
    late.connect(0, &member);
    member.connect(9, &late);
    late.send(0, TransportMessage::Ping { from: 9 });
    member.send(9, TransportMessage::Ready { from: 0 });

    // Assert
    assert_eq!(member.recv(), Some(TransportMessage::Ping { from: 9 }));
    assert_eq!(late.recv(), Some(TransportMessage::Ready { from: 0 }));
}

#[test]
fn new_mesh_builds_one_transport_per_peer() {
    // Arrange & Act
    let mesh = InMemoryTransport::new_mesh(&[4, 5, 6]);

    // Assert
    assert_eq!(mesh.len(), 3);
}

#[test]
fn push_inbox_message_is_received_by_its_own_transport() {
    // Arrange
    let mut mesh = InMemoryTransport::new_mesh(&[4, 5]);

    // Act
    mesh[0].push_inbox(TransportMessage::Ping { from: 9 });

    // Assert
    assert_eq!(mesh[0].recv(), Some(TransportMessage::Ping { from: 9 }));
    assert_eq!(mesh[1].recv(), None);
}

#[test]
fn recv_empty_inbox_returns_none() {
    // Arrange
    let mut mesh = InMemoryTransport::new_mesh(&[4, 5]);

    // Act
    let message = mesh[1].recv();

    // Assert
    assert_eq!(message, None);
}

#[test]
fn send_reaches_only_the_addressed_peer() {
    // Arrange
    let mut mesh = InMemoryTransport::new_mesh(&[4, 5, 6]);

    // Act
    mesh[0].send(6, TransportMessage::Ready { from: 4 });

    // Assert
    assert_eq!(mesh[2].recv(), Some(TransportMessage::Ready { from: 4 }));
    assert_eq!(mesh[1].recv(), None);
    assert_eq!(mesh[0].recv(), None);
}
