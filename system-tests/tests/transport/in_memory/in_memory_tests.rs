// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;
use faction_system_tests::transport::in_memory::in_memory_transport::InMemoryTransport;

fn msg_ping(from: PeerId) -> TransportMessage {
    TransportMessage::Ping { from }
}

fn msg_ready(from: PeerId) -> TransportMessage {
    TransportMessage::Ready { from }
}

#[test]
fn send_between_two_peers_received() {
    // Arrange
    let mut transports = InMemoryTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, msg_ping(0));

    // Assert
    assert_eq!(transports[1].recv(), Some(msg_ping(0)));
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn send_to_self_not_delivered() {
    // Arrange
    let mut transports = InMemoryTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(0, msg_ping(0));

    // Assert
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn recv_empty_returns_none() {
    // Arrange
    let mut transports = InMemoryTransport::new_mesh(&[0, 1]);

    // Act & Assert
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn send_fifo_order_preserved() {
    // Arrange
    let mut transports = InMemoryTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, msg_ping(0));
    transports[0].send(1, msg_ready(0));

    // Assert
    assert_eq!(transports[1].recv(), Some(msg_ping(0)));
    assert_eq!(transports[1].recv(), Some(msg_ready(0)));
}

#[test]
fn send_three_peer_all_delivered() {
    // Arrange
    let mut transports = InMemoryTransport::new_mesh(&[0, 1, 2]);

    // Act
    transports[0].send(1, msg_ping(0));
    transports[0].send(2, msg_ready(0));

    // Assert
    assert_eq!(transports[1].recv(), Some(msg_ping(0)));
    assert_eq!(transports[2].recv(), Some(msg_ready(0)));
}

#[test]
fn send_all_message_types() {
    // Arrange
    let mut transports = InMemoryTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, TransportMessage::Ping { from: 0 });
    transports[0].send(1, TransportMessage::Ready { from: 0 });
    transports[0].send(1, TransportMessage::Bootstrapped { from: 0 });

    // Assert
    assert_eq!(
        transports[1].recv(),
        Some(TransportMessage::Ping { from: 0 })
    );
    assert_eq!(
        transports[1].recv(),
        Some(TransportMessage::Ready { from: 0 })
    );
    assert_eq!(
        transports[1].recv(),
        Some(TransportMessage::Bootstrapped { from: 0 })
    );
}
