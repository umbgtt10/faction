// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction::reconfiguration::Reconfiguration;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol_validation::cluster::Cluster;

const NEWCOMER: u64 = 3;
const ROUNDS: usize = 100;

// Three nodes requiring two, concluded, with a newcomer waiting to be added.
fn concluded_with_a_newcomer() -> Cluster {
    let mut cluster = Cluster::new(3, 2, 1);
    cluster.start_all();
    cluster.run(ROUNDS);
    cluster.add_newcomer(NEWCOMER);
    cluster
}

fn membership_from_node_0() -> TransportMessage {
    TransportMessage::Membership {
        from: 0,
        members: vec![0, 1, 2],
        required_count: 2,
    }
}

#[test]
fn a_commit_for_another_peer_leaves_the_addition_in_flight() {
    // Arrange
    let mut cluster = concluded_with_a_newcomer();
    cluster.request_addition(0, NEWCOMER);

    // Act
    cluster.commit_addition(0, 4);

    // Assert
    assert_eq!(
        cluster.reconfiguration(0),
        Reconfiguration::AdditionRequested { peer_id: NEWCOMER }
    );
    assert_eq!(cluster.members(0), vec![0, 1, 2, 4]);
}

#[test]
fn a_commit_reaching_nodes_still_bootstrapping_admits_the_peer_and_counts_it() {
    // Arrange
    let mut cluster = Cluster::new(3, 4, 1);
    for index in 0..3 {
        cluster.commit_addition(index, NEWCOMER);
    }
    cluster.start_all();

    // Act
    for index in 0..3 {
        cluster.inject_transport(index, TransportMessage::Ready { from: NEWCOMER });
    }
    cluster.run(ROUNDS);

    // Assert
    assert!(cluster.is_bootstrapped());
    assert_eq!(cluster.node_collecting_peers_len(0), 4);
}

#[test]
fn a_lost_acknowledgement_is_recovered_from_the_membership_sent_again() {
    // Arrange
    let mut cluster = concluded_with_a_newcomer();
    cluster.drop_message(
        NEWCOMER,
        0,
        TransportMessage::MembershipAcknowledged { from: NEWCOMER },
        1,
    );
    cluster.request_addition(0, NEWCOMER);

    // Act
    cluster.run(ROUNDS);

    // Assert
    assert_eq!(
        cluster.reconfiguration(0),
        Reconfiguration::AdditionAcknowledged { peer_id: NEWCOMER }
    );
}

#[test]
fn a_lost_membership_is_sent_again_until_acknowledged() {
    // Arrange
    let mut cluster = concluded_with_a_newcomer();
    cluster.drop_message(0, NEWCOMER, membership_from_node_0(), 1);
    cluster.request_addition(0, NEWCOMER);

    // Act
    cluster.run(ROUNDS);

    // Assert
    assert!(!cluster.is_newcomer_waiting());
    assert_eq!(
        cluster.reconfiguration(0),
        Reconfiguration::AdditionAcknowledged { peer_id: NEWCOMER }
    );
}

#[test]
fn a_required_count_lowered_while_collecting_concludes() {
    // Arrange
    let mut cluster = Cluster::new(3, 3, 1);
    cluster.start_node(0);
    while cluster.node_state(0) != PeerState::Collecting {
        cluster.step_timer_node(0);
    }

    // Act
    cluster.change_required_count(0, 1);

    // Assert
    assert_eq!(cluster.node_state(0), PeerState::Bootstrapped);
}

#[test]
fn an_abort_before_the_acknowledgement_stops_the_sending() {
    // Arrange
    let mut cluster = concluded_with_a_newcomer();
    cluster.drop_message(0, NEWCOMER, membership_from_node_0(), 1);
    cluster.request_addition(0, NEWCOMER);

    // Act
    cluster.abort_addition(0, NEWCOMER);
    cluster.run(ROUNDS);

    // Assert
    assert!(cluster.is_newcomer_waiting());
    assert_eq!(cluster.reconfiguration(0), Reconfiguration::None);
}
