// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Removal in the deterministic cluster: the test plays the layer that commits,
// reporting a removal to each node it names; time passes for a node's
// liveness only when a test says so.

use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol_validation::cluster::Cluster;

const NEWCOMER: u64 = 3;
const ROUNDS: usize = 100;

// The removal of `peer` reported committed to every node the cluster began
// with.
fn committed_everywhere(cluster: Cluster, peer: u64) -> Cluster {
    let mut cluster = cluster;
    for index in 0..3 {
        cluster.commit_removal(index, peer);
    }
    cluster.run(ROUNDS);
    cluster
}

fn concluded(size: usize, required: usize) -> Cluster {
    let mut cluster = Cluster::new(size, required, 1);
    cluster.start_all();
    cluster.run(ROUNDS);
    cluster
}

// Node 0's periods passed until it has probed `peer`, and `peer` has missed
// that probe, directly and through the helpers, every answer of its to node 0
// lost: at a threshold of one, node 0 suspects it.
fn suspected_by_node_0(cluster: Cluster, peer: u64) -> Cluster {
    let mut cluster = cluster;
    cluster.drop_message(
        peer,
        0,
        TransportMessage::ProbeAnswer {
            from: peer,
            relay_for: None,
        },
        usize::MAX,
    );
    for helper in [1, 2].into_iter().filter(|helper| *helper != peer) {
        cluster.drop_message(
            peer,
            helper,
            TransportMessage::ProbeAnswer {
                from: peer,
                relay_for: Some(0),
            },
            usize::MAX,
        );
    }
    for _ in 0..4 {
        cluster.elapse(0, TimerMessage::ProbeDue);
        cluster.run(ROUNDS);
        if cluster.last_probed(0) == Some(peer) {
            break;
        }
    }
    cluster.elapse(0, TimerMessage::ProbeMissed { peer_id: peer });
    cluster.run(ROUNDS);
    cluster.elapse(0, TimerMessage::IndirectProbesMissed { peer_id: peer });
    cluster.run(ROUNDS);
    cluster
}

#[test]
fn a_commit_loses_the_quorum_on_a_node_that_still_hears_the_member() {
    // Arrange
    let mut cluster = suspected_by_node_0(concluded(3, 3), 2);
    cluster.request_removal(0, 2);
    let held_by_node_1 = cluster.quorum(1);

    // Act
    let mut cluster = committed_everywhere(cluster, 2);

    // Assert
    assert_eq!(held_by_node_1, Quorum::Held);
    assert_eq!(cluster.quorum(1), Quorum::Lost);
    assert_eq!(cluster.members(1), vec![0, 1]);
}

#[test]
fn a_node_removed_stops_probing_and_the_rest_no_longer_probe_it() {
    // Arrange
    let mut cluster = committed_everywhere(concluded(3, 2), 2);

    // Act
    cluster.elapse(2, TimerMessage::ProbeDue);
    cluster.run(ROUNDS);
    let probes: Vec<Option<u64>> = (0..2)
        .map(|_| {
            cluster.elapse(0, TimerMessage::ProbeDue);
            cluster.run(ROUNDS);
            cluster.last_probed(0)
        })
        .collect();

    // Assert
    assert_eq!(cluster.node_state(2), PeerState::Removed);
    assert!(!cluster.is_held(2, &TimerMessage::ProbeDue));
    assert_eq!(probes, vec![Some(1), Some(1)]);
}

#[test]
fn a_removal_committed_while_bootstrapping_with_a_lowered_count_lets_the_rest_conclude() {
    // Arrange
    let mut cluster = Cluster::new(3, 3, 1);
    cluster.start_node(0);
    cluster.start_node(1);
    cluster.run(ROUNDS);
    let concluded_before = cluster.is_bootstrapped();

    // Act
    for index in 0..2 {
        cluster.change_required_count(index, 2);
        cluster.commit_removal(index, 2);
    }
    cluster.run(ROUNDS);

    // Assert
    assert!(!concluded_before);
    for index in 0..2 {
        assert_eq!(cluster.node_state(index), PeerState::Bootstrapped);
        assert_eq!(cluster.members(index), vec![0, 1]);
    }
}

#[test]
fn a_removal_in_flight_refuses_an_addition_and_an_addition_in_flight_a_removal() {
    // Arrange
    let mut cluster = concluded(3, 2);
    cluster.add_newcomer(NEWCOMER);
    cluster.request_removal(0, 1);
    cluster.request_addition(0, NEWCOMER);
    let while_removing = cluster.reconfiguration(0);
    cluster.abort_removal(0, 1);
    cluster.request_addition(0, NEWCOMER);

    // Act
    cluster.request_removal(0, 1);

    // Assert
    assert_eq!(
        while_removing,
        Reconfiguration::RemovalRequested { peer_id: 1 }
    );
    assert_eq!(
        cluster.reconfiguration(0),
        Reconfiguration::AdditionRequested { peer_id: NEWCOMER }
    );
}

#[test]
fn a_removed_member_s_late_answer_is_ignored() {
    // Arrange
    let mut cluster = suspected_by_node_0(concluded(3, 2), 1);
    cluster.request_removal(0, 1);
    let mut cluster = committed_everywhere(cluster, 1);

    // Act
    cluster.inject_transport(
        0,
        TransportMessage::ProbeAnswer {
            from: 1,
            relay_for: None,
        },
    );
    cluster.run(ROUNDS);

    // Assert
    assert_eq!(cluster.members(0), vec![0, 2]);
    assert!(cluster.suspected(0).is_empty());
}

#[test]
fn a_suspect_is_removed_and_every_node_shows_the_members_without_it() {
    // Arrange
    let mut cluster = suspected_by_node_0(concluded(3, 2), 1);
    let suspected = cluster.suspected(0);

    // Act
    cluster.request_removal(0, 1);
    let mut cluster = committed_everywhere(cluster, 1);

    // Assert
    assert_eq!(suspected, vec![1]);
    assert_eq!(cluster.members(0), vec![0, 2]);
    assert_eq!(cluster.members(2), vec![0, 2]);
    assert_eq!(cluster.node_state(1), PeerState::Removed);
    assert_eq!(cluster.quorum(0), Quorum::Held);
    assert_eq!(cluster.reconfiguration(0), Reconfiguration::None);
}

#[test]
fn an_abort_withdraws_the_removal_and_the_member_stays_on_every_node() {
    // Arrange
    let mut cluster = concluded(3, 2);
    cluster.request_removal(0, 1);

    // Act
    cluster.abort_removal(0, 1);

    // Assert
    assert_eq!(cluster.reconfiguration(0), Reconfiguration::None);
    for index in 0..3 {
        assert_eq!(cluster.members(index), vec![0, 1, 2]);
    }
}
