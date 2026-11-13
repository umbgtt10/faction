// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::timer::real::thread_pause::ThreadPause;
use faction_system_tests::infra::transport::tcp::connection::Connection;
use faction_system_tests::infra::transport::wire_codec::WireCodec;
use faction_system_tests::traits::pause::Pause;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::yield_now;
use std::time::Duration;

const MORE_THAN_A_SOCKET_HOLDS: usize = 16 * 1024 * 1024;

// A connected pair: the writing end, non-blocking, and the reading end.
fn connected() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let writer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (reader, _) = listener.accept().unwrap();
    writer.set_nonblocking(true).unwrap();
    (writer, reader)
}

// A pause that drains the far end of the connection while the writer waits,
// counting how often it was asked to wait.
struct DrainingPause {
    reader: Mutex<TcpStream>,
    pauses: AtomicUsize,
}

impl Pause for DrainingPause {
    fn pause(&self, _duration: Duration) {
        self.pauses.fetch_add(1, Ordering::Relaxed);
        let mut drained = vec![0u8; 1024 * 1024];
        let _ = self.reader.lock().unwrap().read(&mut drained);
    }
}

#[test]
fn messages_of_two_frames_delivered_at_once_returns_both_in_order() {
    // Arrange
    let (writer, mut reader) = connected();
    let frames = [
        WireCodec::encode(&TransportMessage::Ping { from: 1 }),
        WireCodec::encode(&TransportMessage::Ready { from: 2 }),
    ]
    .concat();
    reader.write_all(&frames).unwrap();
    let mut connection = Connection::new(writer);

    // Act
    let mut messages = Vec::new();
    while messages.len() < 2 {
        connection.fill();
        messages.extend(connection.messages());
        yield_now();
    }

    // Assert
    assert_eq!(
        messages,
        vec![
            TransportMessage::Ping { from: 1 },
            TransportMessage::Ready { from: 2 },
        ]
    );
}

#[test]
fn write_of_more_than_a_socket_holds_waits_and_completes() {
    // Arrange
    let (writer, reader) = connected();
    reader.set_nonblocking(true).unwrap();
    let pause = DrainingPause {
        reader: Mutex::new(reader),
        pauses: AtomicUsize::new(0),
    };
    let mut connection = Connection::new(writer);

    // Act
    let written = connection.write(&vec![0u8; MORE_THAN_A_SOCKET_HOLDS], &pause);

    // Assert
    assert!(written);
    assert!(pause.pauses.load(Ordering::Relaxed) > 0);
}

#[test]
fn write_to_a_peer_that_has_closed_returns_false() {
    // Arrange
    let (writer, reader) = connected();
    drop(reader);
    let mut connection = Connection::new(writer);

    // Act
    let written = connection.write(&vec![0u8; MORE_THAN_A_SOCKET_HOLDS], &ThreadPause);

    // Assert
    assert!(!written);
}
