// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::transport::grpc::grpc_transport::GrpcTransport;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::thread::yield_now;

fn free_addr() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap()
}

fn msg_ping(from: PeerId) -> TransportMessage {
    TransportMessage::Ping { from }
}

fn msg_ready(from: PeerId) -> TransportMessage {
    TransportMessage::Ready { from }
}

fn received(transport: &mut impl Transport) -> Option<TransportMessage> {
    loop {
        if let Some(message) = transport.recv() {
            return Some(message);
        }
        yield_now();
    }
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

#[test]
fn fifo_order_preserved() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, msg_ping(0));
    transports[0].send(1, msg_ready(0));

    // Assert
    assert_eq!(received(&mut transports[1]), Some(msg_ping(0)));
    assert_eq!(received(&mut transports[1]), Some(msg_ready(0)));
}

#[test]
fn join_mesh_late_peer_receives_from_the_mesh() {
    // Arrange
    let mut mesh = GrpcTransport::new_mesh(&[0, 1]);
    let mut late = GrpcTransport::join_mesh(2, mesh[0].registry());

    // Act
    mesh[0].send(2, msg_ping(0));

    // Assert
    assert_eq!(received(&mut late), Some(msg_ping(0)));
}

#[test]
fn new_before_its_peer_listens_delivers_once_the_peer_does() {
    // Arrange
    let addrs = [free_addr(), free_addr()];
    let peers = [(0, addrs[0]), (1, addrs[1])];
    let mut early = GrpcTransport::new(addrs[0], &peers);
    let mut late = GrpcTransport::new(addrs[1], &peers);

    // Act
    early.send(1, msg_ping(0));

    // Assert
    assert_eq!(received(&mut late), Some(msg_ping(0)));
}

#[test]
fn recv_empty_returns_none() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1]);

    // Act & Assert
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn registry_lists_every_peer_of_the_mesh() {
    // Arrange
    let mesh = GrpcTransport::new_mesh(&[4, 5]);

    // Act
    let registry = mesh[0].registry();

    // Assert
    let peers: Vec<PeerId> = registry
        .lock()
        .unwrap()
        .iter()
        .map(|(peer, _)| *peer)
        .collect();
    assert_eq!(peers, vec![4, 5]);
}

#[test]
fn send_all_message_types() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, TransportMessage::Ping { from: 0 });
    transports[0].send(1, TransportMessage::Ready { from: 0 });
    transports[0].send(1, TransportMessage::Bootstrapped { from: 0 });

    // Assert
    assert_eq!(
        received(&mut transports[1]),
        Some(TransportMessage::Ping { from: 0 })
    );
    assert_eq!(
        received(&mut transports[1]),
        Some(TransportMessage::Ready { from: 0 })
    );
    assert_eq!(
        received(&mut transports[1]),
        Some(TransportMessage::Bootstrapped { from: 0 })
    );
}

#[test]
fn send_and_recv_between_two_peers() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1]);

    // Act
    transports[0].send(1, msg_ping(0));

    // Assert
    assert_eq!(received(&mut transports[1]), Some(msg_ping(0)));
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn send_three_peer_all_deliver() {
    // Arrange
    let mut transports = GrpcTransport::new_mesh(&[0, 1, 2]);

    // Act
    transports[0].send(1, msg_ping(0));
    transports[0].send(2, msg_ready(0));

    // Assert
    assert_eq!(received(&mut transports[1]), Some(msg_ping(0)));
    assert_eq!(received(&mut transports[2]), Some(msg_ready(0)));
}
