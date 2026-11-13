// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::logging::shared_writer::SharedWriter;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;
use faction_system_tests::infra::transport::faulty::faulty_transport::FaultyTransport;
use faction_system_tests::infra::transport::faulty::link_cuts::LinkCuts;
use faction_system_tests::infra::transport::faulty::message_kind::MessageKind;
use faction_system_tests::infra::transport::in_memory::in_memory_transport::InMemoryTransport;
use std::env::temp_dir;
use std::fs::read_to_string;
use std::fs::remove_file;
use std::process::id;
use std::sync::Arc;
use std::sync::Mutex;

fn delivered_count(policy: FaultPolicy) -> usize {
    let (mut transport, forwarded) = faulty(0, policy);
    for _ in 0..100 {
        transport.send(1, ping(0));
    }
    forwarded.lock().unwrap().len()
}

fn faulty(peer_id: PeerId, policy: FaultPolicy) -> (FaultyTransport, Forwarded) {
    let forwarded: Forwarded = Arc::new(Mutex::new(Vec::new()));
    let inner = RecordingTransport {
        forwarded: forwarded.clone(),
    };
    (
        FaultyTransport::new(peer_id, policy, LinkCuts::default(), None, Box::new(inner)),
        forwarded,
    )
}

type Forwarded = Arc<Mutex<Vec<(PeerId, TransportMessage)>>>;

fn ping(from: PeerId) -> TransportMessage {
    TransportMessage::Ping { from }
}

struct RecordingTransport {
    forwarded: Forwarded,
}

impl Transport for RecordingTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        self.forwarded.lock().unwrap().push((to, message));
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        None
    }
}

#[test]
fn asymmetric_at_100_percent_cuts_outgoing() {
    // Arrange
    let policy = FaultPolicy {
        asymmetric: 100,
        ..FaultPolicy::default()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));

    // Assert
    assert!(forwarded.lock().unwrap().is_empty());
}

#[test]
fn delay_at_100_percent_holds_then_releases() {
    // Arrange
    let policy = FaultPolicy {
        delay: 100,
        ..FaultPolicy::default()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));
    let held_before_pump = forwarded.lock().unwrap().is_empty();
    transport.recv();
    transport.recv();
    transport.recv();

    // Assert
    assert!(held_before_pump);
    assert_eq!(forwarded.lock().unwrap().len(), 1);
}

#[test]
fn duplication_at_100_percent_forwards_twice() {
    // Arrange
    let policy = FaultPolicy {
        duplication: 100,
        ..FaultPolicy::default()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));

    // Assert
    assert_eq!(forwarded.lock().unwrap().len(), 2);
}

#[test]
fn loss_at_100_percent_drops_every_message() {
    // Arrange
    let policy = FaultPolicy {
        loss: 100,
        ..FaultPolicy::default()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));
    transport.send(1, ping(0));

    // Assert
    assert!(forwarded.lock().unwrap().is_empty());
}

#[test]
fn loss_is_deterministic_for_a_fixed_seed() {
    // Arrange
    let policy = FaultPolicy {
        loss: 50,
        seed: 12345,
        ..FaultPolicy::default()
    };

    // Act
    let first = delivered_count(policy);
    let second = delivered_count(policy);

    // Assert
    assert_eq!(first, second);
    assert!(first > 0 && first < 100);
}

#[test]
fn none_policy_forwards_every_message() {
    // Arrange
    let (mut transport, forwarded) = faulty(0, FaultPolicy::default());

    // Act
    transport.send(1, ping(0));
    transport.send(1, TransportMessage::Ready { from: 0 });

    // Assert
    assert_eq!(forwarded.lock().unwrap().len(), 2);
}

#[test]
fn partition_at_100_percent_cuts_the_link() {
    // Arrange
    let policy = FaultPolicy {
        partition: 100,
        ..FaultPolicy::default()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));

    // Assert
    assert!(forwarded.lock().unwrap().is_empty());
}

#[test]
fn recv_from_a_peer_cut_off_drops_what_it_sent() {
    // Arrange
    let mut mesh = InMemoryTransport::new_mesh(&[0, 1]);
    let mut peer_one = mesh.remove(1);
    let cuts = LinkCuts::default();
    let mut faulty = FaultyTransport::new(
        0,
        FaultPolicy::default(),
        cuts.clone(),
        None,
        Box::new(mesh.remove(0)),
    );
    cuts.cut(1);
    peer_one.send(0, ping(1));

    // Act
    let received = faulty.recv();

    // Assert
    assert_eq!(received, None);
}

#[test]
fn reorder_at_100_percent_holds_then_eventually_delivers() {
    // Arrange
    let policy = FaultPolicy {
        reorder: 100,
        ..FaultPolicy::default()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));
    for _ in 0..5 {
        transport.recv();
    }

    // Assert
    assert_eq!(forwarded.lock().unwrap().len(), 1);
}

#[test]
fn selective_at_100_percent_drops_only_the_target_kind() {
    // Arrange
    let policy = FaultPolicy {
        selective: 100,
        selective_target: MessageKind::Ready,
        ..FaultPolicy::default()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, TransportMessage::Ping { from: 0 });
    transport.send(1, TransportMessage::Ready { from: 0 });
    transport.send(1, TransportMessage::Bootstrapped { from: 0 });

    // Assert
    let forwarded = forwarded.lock().unwrap();
    assert_eq!(forwarded.len(), 2);
    assert!(
        !forwarded
            .iter()
            .any(|(_, message)| matches!(message, TransportMessage::Ready { .. }))
    );
}

#[test]
fn send_to_a_peer_cut_off_drops_it_and_logs_the_fault() {
    // Arrange
    let path = temp_dir().join(format!("faulty-transport-tests-{}.jsonl", id()));
    let _ = remove_file(&path);
    let forwarded: Forwarded = Arc::new(Mutex::new(Vec::new()));
    let cuts = LinkCuts::default();
    let mut faulty = FaultyTransport::new(
        0,
        FaultPolicy::default(),
        cuts.clone(),
        Some(SharedWriter::open(&path)),
        Box::new(RecordingTransport {
            forwarded: forwarded.clone(),
        }),
    );
    cuts.cut(1);

    // Act
    faulty.send(1, ping(0));
    faulty.send(2, ping(0));

    // Assert
    assert_eq!(*forwarded.lock().unwrap(), vec![(2, ping(0))]);
    let log = read_to_string(&path).unwrap();
    assert!(
        log.contains("\"event\":\"fault\"") && log.contains("\"to\":1"),
        "{log}"
    );
}
