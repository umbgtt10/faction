// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;
use faction_system_tests::transport::tcp::tcp_transport::TcpTransport;

fn msg_ping(from: PeerId) -> TransportMessage {
    TransportMessage::Ping { from }
}

fn msg_ready(from: PeerId) -> TransportMessage {
    TransportMessage::Ready { from }
}

#[test]
fn mesh_send_and_recv_between_two_peers() {
    // Arrange
    let mut transports = TcpTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, msg_ping(0));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Assert
    assert_eq!(transports[1].recv(), Some(msg_ping(0)));
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn mesh_fifo_order_preserved() {
    // Arrange
    let mut transports = TcpTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, msg_ping(0));
    transports[0].send(1, msg_ready(0));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Assert
    assert_eq!(transports[1].recv(), Some(msg_ping(0)));
    assert_eq!(transports[1].recv(), Some(msg_ready(0)));
}

#[test]
fn mesh_recv_empty_returns_none() {
    // Arrange
    let mut transports = TcpTransport::new_mesh(&[0, 1]);

    // Act & Assert
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn mesh_three_peer_all_deliver() {
    // Arrange
    let mut transports = TcpTransport::new_mesh(&[0, 1, 2]);

    // Act
    transports[0].send(1, msg_ping(0));
    transports[0].send(2, msg_ready(0));
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Assert
    assert_eq!(transports[1].recv(), Some(msg_ping(0)));
    assert_eq!(transports[2].recv(), Some(msg_ready(0)));
}

#[test]
fn mesh_send_all_message_types() {
    // Arrange
    let mut transports = TcpTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, TransportMessage::Ping { from: 0 });
    transports[0].send(1, TransportMessage::Ready { from: 0 });
    transports[0].send(1, TransportMessage::Bootstrapped { from: 0 });
    std::thread::sleep(std::time::Duration::from_millis(50));

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
fn mesh_drop_does_not_hang() {
    // Arrange & Act & Assert
    let transports = TcpTransport::new_mesh(&[0, 1]);
    drop(transports);
}

#[test]
fn mesh_drop_releases_ports() {
    // Arrange
    let transports = TcpTransport::new_mesh(&[0, 1]);

    // Act
    drop(transports);

    // Assert — new mesh should not have port conflicts
    let _transports2 = TcpTransport::new_mesh(&[0, 1]);
}
