// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::thread::sleep;
use std::time::Duration;

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;
use faction_system_tests::transport::grpc::grpc_transport::GrpcTransport;

fn msg_ping(from: PeerId) -> TransportMessage {
    TransportMessage::Ping { from }
}

fn msg_ready(from: PeerId) -> TransportMessage {
    TransportMessage::Ready { from }
}

#[test]
fn send_and_recv_between_two_peers() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, msg_ping(0));
    sleep(Duration::from_millis(100));

    // Assert
    assert_eq!(transports[1].recv(), Some(msg_ping(0)));
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn fifo_order_preserved() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, msg_ping(0));
    transports[0].send(1, msg_ready(0));
    sleep(Duration::from_millis(100));

    // Assert
    assert_eq!(transports[1].recv(), Some(msg_ping(0)));
    assert_eq!(transports[1].recv(), Some(msg_ready(0)));
}

#[test]
fn recv_empty_returns_none() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1]);

    // Act & Assert
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn send_three_peer_all_deliver() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1, 2]);

    // Act
    transports[0].send(1, msg_ping(0));
    transports[0].send(2, msg_ready(0));
    sleep(Duration::from_millis(100));

    // Assert
    assert_eq!(transports[1].recv(), Some(msg_ping(0)));
    assert_eq!(transports[2].recv(), Some(msg_ready(0)));
}

#[test]
fn send_all_message_types() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, TransportMessage::Ping { from: 0 });
    transports[0].send(1, TransportMessage::Ready { from: 0 });
    transports[0].send(1, TransportMessage::Bootstrapped { from: 0 });
    sleep(Duration::from_millis(100));

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

#[test]
fn drop_does_not_hang() {
    // Arrange & Act & Assert — drop at end of scope
    let transports = GrpcTransport::new_mesh(&[0, 1]);
    drop(transports);
}

#[test]
fn drop_releases_server_port() {
    // Arrange
    let transports = GrpcTransport::new_mesh(&[0, 1]);

    // Act — drop frees the server
    drop(transports);

    // Assert — creating a new mesh on same ports should not conflict
    let _transports2 = GrpcTransport::new_mesh(&[0, 1]);
}
