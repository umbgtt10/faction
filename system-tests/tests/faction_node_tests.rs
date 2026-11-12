// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::fake_time;
use crate::traits::monotonic_clock_tests::FakeMonotonicClock;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;
use faction_protocol::protocol::Protocol;
use faction_system_tests::faction_node::FactionNode;
use faction_system_tests::infra::timer::real::real_timer::RealTimer;
use faction_system_tests::infra::transport::in_memory::in_memory_transport::InMemoryTransport;
use faction_system_tests::no_op_node_observer::NoOpNodeObserver;
use faction_system_tests::node_snapshot::NodeSnapshot;
use std::sync::Arc;
use std::time::Duration;

fn node() -> (FactionNode, Arc<FakeMonotonicClock>) {
    let (time, clock) = fake_time();
    let peers = vec![0, 1, 2];
    let config = Config::new(0, peers.clone(), QuorumPolicy::new(2));
    let protocol = Protocol::new(
        Faction::new(config, Box::new(NoOpObserver)),
        peers.clone(),
        0,
    );
    let timer = RealTimer::with_delays(
        time.clock.clone(),
        Duration::from_millis(10),
        Duration::from_secs(10),
    );
    let transport = InMemoryTransport::new_mesh(&peers).remove(0);
    let node = FactionNode::new(
        0,
        peers,
        protocol,
        Box::new(transport),
        Box::new(timer),
        Box::new(NoOpNodeObserver),
    );
    (node, clock)
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
fn deny_peer_leaves_the_members_as_they_were() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.deny(7);

    // Assert
    assert_eq!(node.member_count(), 3);
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
fn new_starts_fresh() {
    // Arrange & Act
    let (mut node, _) = node();

    // Assert
    assert_eq!(
        node.snapshot(),
        NodeSnapshot {
            peer_state: PeerState::Fresh,
            member_count: 3,
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
fn request_join_leaves_the_members_as_they_were() {
    // Arrange
    let (mut node, _) = node();

    // Act
    node.request_join(7);

    // Assert
    assert_eq!(node.member_count(), 3);
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
fn step_with_nothing_due_returns_false() {
    // Arrange
    let (mut node, _) = node();

    // Act
    let had_work = node.step();

    // Assert
    assert!(!had_work);
    assert_eq!(node.peer_state(), PeerState::Fresh);
}
