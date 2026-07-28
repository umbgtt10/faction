// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::sync::Mutex;

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;
use faction_system_tests::transport::faulty::fault_policy::FaultPolicy;
use faction_system_tests::transport::faulty::faulty_transport::FaultyTransport;
use faction_system_tests::transport::faulty::message_kind::MessageKind;

type Forwarded = Arc<Mutex<Vec<(PeerId, TransportMessage)>>>;

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

fn faulty(peer_id: PeerId, policy: FaultPolicy) -> (FaultyTransport, Forwarded) {
    let forwarded: Forwarded = Arc::new(Mutex::new(Vec::new()));
    let inner = RecordingTransport {
        forwarded: forwarded.clone(),
    };
    (
        FaultyTransport::new(peer_id, policy, Box::new(inner)),
        forwarded,
    )
}

fn ping(from: PeerId) -> TransportMessage {
    TransportMessage::Ping { from }
}

fn delivered_count(policy: FaultPolicy) -> usize {
    let (mut transport, forwarded) = faulty(0, policy);
    for _ in 0..100 {
        transport.send(1, ping(0));
    }
    forwarded.lock().unwrap().len()
}

#[test]
fn none_policy_forwards_every_message() {
    // Arrange
    let (mut transport, forwarded) = faulty(0, FaultPolicy::none());

    // Act
    transport.send(1, ping(0));
    transport.send(1, TransportMessage::Ready { from: 0 });

    // Assert
    assert_eq!(forwarded.lock().unwrap().len(), 2);
}

#[test]
fn loss_at_100_percent_drops_every_message() {
    // Arrange
    let policy = FaultPolicy {
        loss: 100,
        ..FaultPolicy::none()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));
    transport.send(1, ping(0));

    // Assert
    assert!(forwarded.lock().unwrap().is_empty());
}

#[test]
fn duplication_at_100_percent_forwards_twice() {
    // Arrange
    let policy = FaultPolicy {
        duplication: 100,
        ..FaultPolicy::none()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));

    // Assert
    assert_eq!(forwarded.lock().unwrap().len(), 2);
}

#[test]
fn partition_at_100_percent_cuts_the_link() {
    // Arrange
    let policy = FaultPolicy {
        partition: 100,
        ..FaultPolicy::none()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));

    // Assert
    assert!(forwarded.lock().unwrap().is_empty());
}

#[test]
fn asymmetric_at_100_percent_cuts_outgoing() {
    // Arrange
    let policy = FaultPolicy {
        asymmetric: 100,
        ..FaultPolicy::none()
    };
    let (mut transport, forwarded) = faulty(0, policy);

    // Act
    transport.send(1, ping(0));

    // Assert
    assert!(forwarded.lock().unwrap().is_empty());
}

#[test]
fn selective_at_100_percent_drops_only_the_target_kind() {
    // Arrange
    let policy = FaultPolicy {
        selective: 100,
        selective_target: MessageKind::Ready,
        ..FaultPolicy::none()
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
fn delay_at_100_percent_holds_then_releases() {
    // Arrange
    let policy = FaultPolicy {
        delay: 100,
        ..FaultPolicy::none()
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
fn reorder_at_100_percent_holds_then_eventually_delivers() {
    // Arrange
    let policy = FaultPolicy {
        reorder: 100,
        ..FaultPolicy::none()
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
fn loss_is_deterministic_for_a_fixed_seed() {
    // Arrange
    let policy = FaultPolicy {
        loss: 50,
        seed: 12345,
        ..FaultPolicy::none()
    };

    // Act
    let first = delivered_count(policy);
    let second = delivered_count(policy);

    // Assert
    assert_eq!(first, second);
    assert!(first > 0 && first < 100);
}
