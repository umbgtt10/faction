// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::live_policy::LivePolicy;
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
        FaultyTransport::new(
            peer_id,
            LivePolicy::new(policy),
            LinkCuts::default(),
            None,
            Box::new(inner),
        ),
        forwarded,
    )
}

type Forwarded = Arc<Mutex<Vec<(PeerId, TransportMessage)>>>;

fn live_faulty(policy: FaultPolicy) -> (FaultyTransport, Forwarded, LivePolicy<FaultPolicy>) {
    let forwarded: Forwarded = Arc::new(Mutex::new(Vec::new()));
    let live = LivePolicy::new(policy);
    let inner = RecordingTransport {
        forwarded: forwarded.clone(),
    };
    (
        FaultyTransport::new(0, live.clone(), LinkCuts::default(), None, Box::new(inner)),
        forwarded,
        live,
    )
}

fn ping(from: PeerId) -> TransportMessage {
    TransportMessage::Ping { from, epoch: 0 }
}

fn receiving(policy: FaultPolicy) -> (FaultyTransport, InMemoryTransport, LivePolicy<FaultPolicy>) {
    let mut mesh = InMemoryTransport::new_mesh(&[0, 1]);
    let peer_one = mesh.remove(1);
    let live = LivePolicy::new(policy);
    (
        FaultyTransport::new(
            0,
            live.clone(),
            LinkCuts::default(),
            None,
            Box::new(mesh.remove(0)),
        ),
        peer_one,
        live,
    )
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
    let first = delivered_count(policy.clone());
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
    transport.send(1, TransportMessage::Ready { from: 0, epoch: 0 });

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
        LivePolicy::default(),
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
fn recv_of_a_deaf_node_receives_nothing() {
    // Arrange
    let (mut faulty, mut peer_one, _) = receiving(FaultPolicy {
        deaf: true,
        ..FaultPolicy::default()
    });
    peer_one.send(0, ping(1));

    // Act
    let received = faulty.recv();

    // Assert
    assert_eq!(received, None);
}

#[test]
fn recv_of_a_starved_node_receives_nothing_until_fed() {
    // Arrange
    let (mut faulty, mut peer_one, live) = receiving(FaultPolicy {
        starved: true,
        ..FaultPolicy::default()
    });
    peer_one.send(0, ping(1));
    let starved = faulty.recv();
    live.set(FaultPolicy::default());

    // Act
    let fed = faulty.recv();

    // Assert
    assert_eq!(starved, None);
    assert_eq!(fed, Some(ping(1)));
}

#[test]
fn recv_past_a_full_inbox_drops_the_tail() {
    // Arrange
    let (mut faulty, mut peer_one, _) = receiving(FaultPolicy {
        inbox: 2,
        ..FaultPolicy::default()
    });
    for epoch in 1..=3 {
        peer_one.send(0, TransportMessage::Ping { from: 1, epoch });
    }

    // Act
    let received: Vec<Option<TransportMessage>> = (0..3).map(|_| faulty.recv()).collect();

    // Assert
    assert_eq!(
        received,
        vec![
            Some(TransportMessage::Ping { from: 1, epoch: 1 }),
            Some(TransportMessage::Ping { from: 1, epoch: 2 }),
            None,
        ]
    );
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
    transport.send(1, TransportMessage::Ping { from: 0, epoch: 0 });
    transport.send(1, TransportMessage::Ready { from: 0, epoch: 0 });
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
fn send_after_its_policy_is_healed_forwards_again() {
    // Arrange
    let forwarded: Forwarded = Arc::new(Mutex::new(Vec::new()));
    let live = LivePolicy::new(FaultPolicy {
        loss: 100,
        ..FaultPolicy::default()
    });
    let mut faulty = FaultyTransport::new(
        0,
        live.clone(),
        LinkCuts::default(),
        None,
        Box::new(RecordingTransport {
            forwarded: forwarded.clone(),
        }),
    );
    faulty.send(1, ping(0));
    let lost = forwarded.lock().unwrap().len();
    live.set(FaultPolicy::default());

    // Act
    faulty.send(1, ping(0));

    // Assert
    assert_eq!(lost, 0);
    assert_eq!(*forwarded.lock().unwrap(), vec![(1, ping(0))]);
}

#[test]
fn send_misdelivered_reaches_another_peer_sent_to_before() {
    // Arrange
    let (mut faulty, forwarded, _) = live_faulty(FaultPolicy {
        misdelivery: 100,
        ..FaultPolicy::default()
    });
    faulty.send(2, ping(0));

    // Act
    faulty.send(1, ping(0));

    // Assert
    assert_eq!(*forwarded.lock().unwrap(), vec![(2, ping(0)), (2, ping(0))]);
}

#[test]
fn send_of_a_mute_node_forwards_nothing() {
    // Arrange
    let (mut faulty, forwarded, _) = live_faulty(FaultPolicy {
        mute: true,
        ..FaultPolicy::default()
    });

    // Act
    faulty.send(1, ping(0));

    // Assert
    assert!(forwarded.lock().unwrap().is_empty());
}

#[test]
fn send_of_a_starved_node_holds_its_sends_until_fed() {
    // Arrange
    let (mut faulty, forwarded, live) = live_faulty(FaultPolicy {
        starved: true,
        ..FaultPolicy::default()
    });
    faulty.send(1, ping(0));
    let starved = forwarded.lock().unwrap().len();
    live.set(FaultPolicy::default());

    // Act
    faulty.send(2, ping(0));

    // Assert
    assert_eq!(starved, 0);
    assert_eq!(*forwarded.lock().unwrap(), vec![(1, ping(0)), (2, ping(0))]);
}

#[test]
fn send_of_the_first_contacts_with_a_peer_loses_them() {
    // Arrange
    let (mut faulty, forwarded, _) = live_faulty(FaultPolicy {
        first_contact_lost: 2,
        ..FaultPolicy::default()
    });

    // Act
    for epoch in 1..=3 {
        faulty.send(1, TransportMessage::Ping { from: 0, epoch });
    }

    // Assert
    assert_eq!(
        *forwarded.lock().unwrap(),
        vec![(1, TransportMessage::Ping { from: 0, epoch: 3 })]
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
        LivePolicy::default(),
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

#[test]
fn send_to_an_unreachable_peer_loses_that_send_alone() {
    // Arrange
    let (mut faulty, forwarded, _) = live_faulty(FaultPolicy {
        unreachable: vec![2],
        ..FaultPolicy::default()
    });

    // Act
    faulty.send(2, ping(0));
    faulty.send(1, ping(0));

    // Assert
    assert_eq!(*forwarded.lock().unwrap(), vec![(1, ping(0))]);
}

#[test]
fn send_to_an_unreachable_peer_under_head_of_line_wedges_every_send_after_it() {
    // Arrange
    let (mut faulty, forwarded, live) = live_faulty(FaultPolicy {
        unreachable: vec![2],
        head_of_line: true,
        ..FaultPolicy::default()
    });
    faulty.send(2, ping(0));
    faulty.send(1, ping(0));
    let wedged = forwarded.lock().unwrap().len();
    live.set(FaultPolicy {
        head_of_line: true,
        ..FaultPolicy::default()
    });

    // Act
    faulty.send(1, ping(0));

    // Assert
    assert_eq!(wedged, 0);
    assert_eq!(*forwarded.lock().unwrap(), vec![(1, ping(0))]);
}
