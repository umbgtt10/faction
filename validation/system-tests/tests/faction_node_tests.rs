// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::fake_time;
use crate::traits::monotonic_clock_tests::FakeMonotonicClock;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum::Quorum;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction_protocol::newcomer::Newcomer;
use faction_protocol::protocol::Protocol;
use faction_protocol::transport::Transport;
use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::constants::DEFAULT_SUSPICION_THRESHOLD;
use faction_system_tests::constants::HELPERS;
use faction_system_tests::faction_node::FactionNode;
use faction_system_tests::infra::timer::real::real_timer::RealTimer;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;
use faction_system_tests::infra::transport::faulty::faulty_transport::FaultyTransport;
use faction_system_tests::infra::transport::faulty::link_cuts::LinkCuts;
use faction_system_tests::infra::transport::in_memory::in_memory_transport::InMemoryTransport;
use faction_system_tests::no_op_node_observer::NoOpNodeObserver;
use faction_system_tests::node_snapshot::NodeSnapshot;
use faction_system_tests::probe_stretch::ProbeStretch;
use faction_system_tests::role::Role;
use std::iter::from_fn;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

// Node 0 of members 0, 1 and 2, requiring one, concluded; the transports of
// peer 1 and of peer 7, which is no member; and the cuts its transport drops
// messages by.
fn concluded() -> (FactionNode, InMemoryTransport, InMemoryTransport, LinkCuts) {
    let (time, clock) = fake_time();
    let mut mesh = InMemoryTransport::new_mesh(&[0, 1, 2, 7]);
    let outsider = mesh.remove(3);
    let peer_one = mesh.remove(1);
    let peers = vec![0, 1, 2];
    let cuts = LinkCuts::default();
    let config = Config::new(0, peers.clone(), QuorumPolicy::new(1), 1);
    let protocol = Protocol::new(
        Faction::new(config, Box::new(NoOpObserver)),
        peers.clone(),
        0,
        HELPERS,
    );
    let timer = RealTimer::with_delays(
        time.clock.clone(),
        Duration::from_millis(10),
        Duration::from_secs(10),
        ProbeStretch::Minimal.factor(),
    );
    let mut node = FactionNode::new(
        0,
        peers,
        Role::Running(Box::new(protocol)),
        Box::new(FaultyTransport::new(
            0,
            FaultPolicy::default(),
            cuts.clone(),
            None,
            Box::new(mesh.remove(0)),
        )),
        Box::new(timer),
        Box::new(NoOpNodeObserver),
        cuts.clone(),
    );
    node.start();
    while node.peer_state() != PeerState::Bootstrapped {
        clock.advance(Duration::from_millis(10));
        node.step();
    }
    (node, peer_one, outsider, cuts)
}

// Peer 5, waiting to be added, and the transport of node 0 that can reach it.
fn newcomer() -> (FactionNode, InMemoryTransport) {
    let (time, _) = fake_time();
    let mut mesh = InMemoryTransport::new_mesh(&[0, 5]);
    let transport = mesh.remove(1);
    let timer = RealTimer::with_delays(
        time.clock.clone(),
        Duration::from_millis(10),
        Duration::from_secs(10),
        ProbeStretch::Minimal.factor(),
    );
    let node = FactionNode::new(
        5,
        Vec::new(),
        Role::Waiting(Newcomer::new(
            5,
            DEFAULT_SUSPICION_THRESHOLD,
            HELPERS,
            Box::new(NoOpObserver),
        )),
        Box::new(transport),
        Box::new(timer),
        Box::new(NoOpNodeObserver),
        LinkCuts::default(),
    );
    (node, mesh.remove(0))
}

fn node() -> (FactionNode, Arc<FakeMonotonicClock>) {
    let (time, clock) = fake_time();
    let peers = vec![0, 1, 2];
    let config = Config::new(0, peers.clone(), QuorumPolicy::new(2), 1);
    let protocol = Protocol::new(
        Faction::new(config, Box::new(NoOpObserver)),
        peers.clone(),
        0,
        HELPERS,
    );
    let timer = RealTimer::with_delays(
        time.clock.clone(),
        Duration::from_millis(10),
        Duration::from_secs(10),
        ProbeStretch::Minimal.factor(),
    );
    let transport = InMemoryTransport::new_mesh(&peers).remove(0);
    let node = FactionNode::new(
        0,
        peers,
        Role::Running(Box::new(protocol)),
        Box::new(transport),
        Box::new(timer),
        Box::new(NoOpNodeObserver),
        LinkCuts::default(),
    );
    (node, clock)
}

// Node 0 of members 0, 1 and 2, its machine's every command recorded.
fn observed() -> (FactionNode, Arc<Mutex<Vec<Command>>>) {
    let (time, _) = fake_time();
    let commands = Arc::new(Mutex::new(Vec::new()));
    let peers = vec![0, 1, 2];
    let config = Config::new(0, peers.clone(), QuorumPolicy::new(2), 1);
    let protocol = Protocol::new(
        Faction::new(
            config,
            Box::new(Recording {
                commands: commands.clone(),
            }),
        ),
        peers.clone(),
        0,
        HELPERS,
    );
    let timer = RealTimer::with_delays(
        time.clock.clone(),
        Duration::from_millis(10),
        Duration::from_secs(10),
        ProbeStretch::Minimal.factor(),
    );
    let node = FactionNode::new(
        0,
        peers.clone(),
        Role::Running(Box::new(protocol)),
        Box::new(InMemoryTransport::new_mesh(&peers).remove(0)),
        Box::new(timer),
        Box::new(NoOpNodeObserver),
        LinkCuts::default(),
    );
    (node, commands)
}

// Every command a node's machine is given.
struct Recording {
    commands: Arc<Mutex<Vec<Command>>>,
}

impl Observer for Recording {
    fn observe(&mut self, command: &Command, _result: &ProcessResult) {
        self.commands.lock().unwrap().push(*command);
    }
}

#[test]
fn abort_addition_of_a_concluded_node_withdraws_its_addition() {
    // Arrange
    let (mut node, _, _, _) = concluded();
    node.request_addition(7);

    // Act
    node.abort_addition(7);

    // Assert
    assert_eq!(node.snapshot().reconfiguration, Reconfiguration::None);
}

#[test]
fn abort_addition_of_a_node_not_concluded_changes_nothing() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.abort_addition(7);

    // Assert
    assert_eq!(node.snapshot().reconfiguration, Reconfiguration::None);
    assert_eq!(node.member_count(), 3);
}

#[test]
fn abort_removal_of_a_concluded_node_withdraws_its_removal() {
    // Arrange
    let (mut node, _, _, _) = concluded();
    node.request_removal(2);

    // Act
    node.abort_removal(2);

    // Assert
    assert_eq!(node.snapshot().reconfiguration, Reconfiguration::None);
    assert_eq!(node.member_count(), 3);
}

#[test]
fn admit_new_peer_adds_a_member() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.admit(7);

    // Assert
    assert_eq!(node.member_count(), 4);
}

#[test]
fn change_required_count_sets_the_count_in_effect() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.change_required_count(3);

    // Assert
    assert_eq!(node.snapshot().required_count, 3);
}

#[test]
fn commit_addition_admits_the_peer() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.commit_addition(7);

    // Assert
    assert_eq!(node.member_count(), 4);
}

#[test]
fn commit_addition_widens_the_broadcasts_to_the_new_member() {
    // Arrange
    let (mut node, mut peer_one, mut outsider, _) = concluded();
    node.commit_addition(7);
    peer_one.send(0, TransportMessage::Ping { from: 1 });

    // Act
    for _ in 0..4 {
        node.step();
    }

    // Assert
    let received: Vec<TransportMessage> = from_fn(|| outsider.recv()).collect();
    assert!(
        received.contains(&TransportMessage::Ready { from: 0 }),
        "{received:?}"
    );
}

#[test]
fn commit_removal_narrows_the_broadcasts_to_the_members_left() {
    // Arrange
    let (mut node, mut peer_one, mut outsider, _) = concluded();
    node.commit_addition(7);
    node.commit_removal(7);
    peer_one.send(0, TransportMessage::Ping { from: 1 });

    // Act
    for _ in 0..4 {
        node.step();
    }

    // Assert
    let received: Vec<TransportMessage> = from_fn(|| outsider.recv()).collect();
    assert!(
        !received.contains(&TransportMessage::Ready { from: 0 }),
        "{received:?}"
    );
}

#[test]
fn commit_removal_takes_the_member_out() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.commit_removal(2);

    // Assert
    assert_eq!(node.member_count(), 2);
}

#[test]
fn cut_cuts_the_link_in_the_cuts_the_transport_drops_by() {
    // Arrange
    let (mut node, _, _, cuts) = concluded();

    // Act
    node.cut(2);

    // Assert
    assert!(cuts.is_cut(2));
}

#[test]
fn cut_keeps_everything_from_crossing_the_link() {
    // Arrange
    let (mut node, mut peer_one, _, _) = concluded();
    let _ = from_fn(|| peer_one.recv()).count();
    node.cut(1);
    peer_one.send(0, TransportMessage::Ping { from: 1 });

    // Act
    for _ in 0..4 {
        node.step();
    }

    // Assert
    assert_eq!(from_fn(|| peer_one.recv()).count(), 0);
}

#[test]
fn deny_peer_leaves_the_members_as_they_were() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.deny(7);

    // Assert
    assert_eq!(node.member_count(), 3);
}

#[test]
fn deny_reports_the_rejection_to_the_machine() {
    // Arrange
    let (mut node, commands) = observed();

    // Act
    node.deny(7);

    // Assert
    assert!(
        commands
            .lock()
            .unwrap()
            .contains(&Command::JoinRejected { peer_id: 7 })
    );
}

#[test]
fn expire_deadline_while_pinging_reports_timed_out() {
    // Arrange
    let (mut node, clock) = node();
    node.start();
    clock.advance(Duration::from_millis(10));
    let _ = node.step();

    // Act
    node.expire_deadline();

    // Assert
    assert_eq!(node.peer_state(), PeerState::TimedOut);
}

#[test]
fn member_count_counts_the_configured_peers() {
    // Arrange
    let (mut node, _) = node();

    // Act
    let members = node.member_count();

    // Assert
    assert_eq!(members, 3);
}

#[test]
fn mend_lets_messages_cross_the_link_again() {
    // Arrange
    let (mut node, mut peer_one, _, _) = concluded();
    node.cut(1);
    node.mend(1);
    peer_one.send(0, TransportMessage::Ping { from: 1 });

    // Act
    for _ in 0..4 {
        node.step();
    }

    // Assert
    assert!(
        from_fn(|| peer_one.recv())
            .any(|message| matches!(message, TransportMessage::Ready { from: 0 }))
    );
}

#[test]
fn new_starts_fresh() {
    // Arrange & Act
    let (mut node, _) = node();

    // Assert
    assert_eq!(
        node.snapshot(),
        NodeSnapshot {
            peer_state: PeerState::Fresh,
            member_count: 3,
            required_count: 2,
            reconfiguration: Reconfiguration::None,
            suspected: Vec::new(),
            quorum: Quorum::NotTracked,
        }
    );
}

#[test]
fn peer_state_of_a_new_node_is_fresh() {
    // Arrange
    let (mut node, _) = node();

    // Act
    let peer_state = node.peer_state();

    // Assert
    assert_eq!(peer_state, PeerState::Fresh);
}

#[test]
fn request_addition_of_a_concluded_node_sends_the_newcomer_its_membership() {
    // Arrange
    let (mut node, _, mut outsider, _) = concluded();

    // Act
    node.request_addition(7);

    // Assert
    assert_eq!(
        node.snapshot().reconfiguration,
        Reconfiguration::AdditionRequested { peer_id: 7 }
    );
    assert_eq!(
        outsider.recv(),
        Some(TransportMessage::Membership {
            from: 0,
            members: vec![0, 1, 2],
            required_count: 1,
        })
    );
}

#[test]
fn request_addition_of_a_node_not_concluded_changes_nothing() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.request_addition(7);

    // Assert
    assert_eq!(node.snapshot().reconfiguration, Reconfiguration::None);
}

#[test]
fn request_join_leaves_the_members_as_they_were() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.request_join(7);

    // Assert
    assert_eq!(node.member_count(), 3);
}

#[test]
fn request_join_reports_the_request_to_the_machine() {
    // Arrange
    let (mut node, commands) = observed();

    // Act
    node.request_join(7);

    // Assert
    assert!(
        commands
            .lock()
            .unwrap()
            .contains(&Command::JoinRequested { peer_id: 7 })
    );
}

#[test]
fn request_removal_of_a_concluded_node_proposes_it() {
    // Arrange
    let (mut node, _, _, _) = concluded();

    // Act
    node.request_removal(2);

    // Assert
    assert_eq!(
        node.snapshot().reconfiguration,
        Reconfiguration::RemovalRequested { peer_id: 2 }
    );
}

#[test]
fn snapshot_after_admitting_reports_the_new_member() {
    // Arrange
    let (mut node, _) = node();
    node.admit(9);

    // Act
    let snapshot = node.snapshot();

    // Assert
    assert_eq!(snapshot.member_count, 4);
    assert_eq!(snapshot.peer_state, PeerState::Fresh);
}

#[test]
fn snapshot_of_a_waiting_newcomer_is_fresh_with_no_members() {
    // Arrange
    let (mut newcomer, _) = newcomer();

    // Act
    let snapshot = newcomer.snapshot();

    // Assert
    assert_eq!(
        snapshot,
        NodeSnapshot {
            peer_state: PeerState::Fresh,
            member_count: 0,
            required_count: 0,
            reconfiguration: Reconfiguration::None,
            suspected: Vec::new(),
            quorum: Quorum::NotTracked,
        }
    );
}

#[test]
fn start_schedules_participation_that_a_due_step_applies() {
    // Arrange
    let (mut node, clock) = node();

    // Act
    node.start();
    clock.advance(Duration::from_millis(10));
    let had_work = node.step();

    // Assert
    assert!(had_work);
    assert_eq!(node.peer_state(), PeerState::Pinging);
}

#[test]
fn step_of_a_waiting_newcomer_given_anything_else_keeps_waiting() {
    // Arrange
    let (mut newcomer, mut coordinator) = newcomer();
    coordinator.send(5, TransportMessage::Ping { from: 0 });

    // Act
    let worked = [newcomer.step(), newcomer.step()];

    // Assert
    assert!(worked.contains(&true));
    assert_eq!(newcomer.member_count(), 0);
}

#[test]
fn step_of_a_waiting_newcomer_given_its_membership_becomes_a_node_of_it() {
    // Arrange
    let (mut newcomer, mut coordinator) = newcomer();
    newcomer.start();
    coordinator.send(
        5,
        TransportMessage::Membership {
            from: 0,
            members: vec![0, 1],
            required_count: 2,
        },
    );

    // Act
    let worked = [newcomer.step(), newcomer.step()];

    // Assert
    assert!(worked.contains(&true));
    assert_eq!(newcomer.member_count(), 3);
    assert_eq!(newcomer.snapshot().required_count, 2);
    assert_eq!(
        coordinator.recv(),
        Some(TransportMessage::MembershipAcknowledged { from: 5 })
    );
}

#[test]
fn step_with_nothing_due_returns_false() {
    // Arrange
    let (mut node, _) = node();

    // Act
    let had_work = node.step();

    // Assert
    assert!(!had_work);
    assert_eq!(node.peer_state(), PeerState::Fresh);
}
