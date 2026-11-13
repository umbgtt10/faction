// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::timer::real::thread_pause::ThreadPause;
use faction_system_tests::infra::transport::tcp::tcp_transport::TcpTransport;
use faction_system_tests::infra::transport::wire_codec::WireCodec;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
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
    // Arrange & Act & Assert
    let transports = TcpTransport::new_mesh(&[0, 1], Arc::new(ThreadPause));
    drop(transports);
}

#[test]
fn drop_releases_ports() {
    // Arrange
    let transports = TcpTransport::new_mesh(&[0, 1], Arc::new(ThreadPause));

    // Act
    drop(transports);

    // Assert — new mesh should not have port conflicts
    let _transports2 = TcpTransport::new_mesh(&[0, 1], Arc::new(ThreadPause));
}

#[test]
fn fifo_order_preserved() {
    // Arrange
    let mut transports = TcpTransport::new_mesh(&[0, 1], Arc::new(ThreadPause));

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
    let mut mesh = TcpTransport::new_mesh(&[0, 1], Arc::new(ThreadPause));
    let mut late = TcpTransport::join_mesh(2, mesh[0].registry(), Arc::new(ThreadPause));

    // Act
    mesh[0].send(2, msg_ping(0));

    // Assert
    assert_eq!(received(&mut late), Some(msg_ping(0)));
}

#[test]
fn new_connects_to_a_listening_peer() {
    // Arrange
    let addrs = [free_addr(), free_addr()];
    let mut listening = TcpTransport::new(addrs[1], 1, &[(1, addrs[1])], Arc::new(ThreadPause));
    let mut dialling = TcpTransport::new(
        addrs[0],
        0,
        &[(0, addrs[0]), (1, addrs[1])],
        Arc::new(ThreadPause),
    );

    // Act
    dialling.send(1, msg_ready(0));

    // Assert
    assert_eq!(received(&mut listening), Some(msg_ready(0)));
}

#[test]
fn recv_empty_returns_none() {
    // Arrange
    let mut transports = TcpTransport::new_mesh(&[0, 1], Arc::new(ThreadPause));

    // Act & Assert
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn recv_of_frames_split_across_reads_on_two_connections_returns_each_whole() {
    // Arrange
    let listen_addr = free_addr();
    let mut transport =
        TcpTransport::new(listen_addr, 0, &[(0, listen_addr)], Arc::new(ThreadPause));
    let first = WireCodec::encode(&msg_ping(1));
    let second = WireCodec::encode(&msg_ready(2));
    let mut connection_one = TcpStream::connect(listen_addr).unwrap();
    let mut connection_two = TcpStream::connect(listen_addr).unwrap();
    connection_one.write_all(&first[..6]).unwrap();
    connection_two.write_all(&second[..6]).unwrap();
    for _ in 0..50 {
        assert_eq!(transport.recv(), None, "a frame read before it was whole");
        yield_now();
    }

    // Act
    connection_one.write_all(&first[6..]).unwrap();
    connection_two.write_all(&second[6..]).unwrap();
    let mut messages = vec![received(&mut transport), received(&mut transport)];

    // Assert
    messages.sort_by_key(|message| format!("{message:?}"));
    assert_eq!(messages, vec![Some(msg_ping(1)), Some(msg_ready(2))]);
}

#[test]
fn registry_lists_every_peer_of_the_mesh() {
    // Arrange
    let mesh = TcpTransport::new_mesh(&[4, 5], Arc::new(ThreadPause));

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
    let mut transports = TcpTransport::new_mesh(&[0, 1], Arc::new(ThreadPause));

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
    let mut transports = TcpTransport::new_mesh(&[0, 1], Arc::new(ThreadPause));

    // Act
    transports[0].send(1, msg_ping(0));

    // Assert
    assert_eq!(received(&mut transports[1]), Some(msg_ping(0)));
    assert_eq!(transports[0].recv(), None);
}

#[test]
fn send_three_peer_all_deliver() {
    // Arrange
    let mut transports = TcpTransport::new_mesh(&[0, 1, 2], Arc::new(ThreadPause));

    // Act
    transports[0].send(1, msg_ping(0));
    transports[0].send(2, msg_ready(0));

    // Assert
    assert_eq!(received(&mut transports[1]), Some(msg_ping(0)));
    assert_eq!(received(&mut transports[2]), Some(msg_ready(0)));
}
