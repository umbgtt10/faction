// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Liveness in the deterministic cluster: time passes for node 0 only when a
// test says so, and the answers a test names are lost on their way.

use faction::quorum::Quorum;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol_validation::cluster::Cluster;

const NEWCOMER: u64 = 3;
const ROUNDS: usize = 100;

// Node 1's answers lost `count` times: the direct one to node 0, and the one
// node 2 relays for node 0.
fn answers_of_one_lost(cluster: Cluster, count: usize) -> Cluster {
    let mut cluster = cluster;
    cluster.drop_message(
        1,
        0,
        TransportMessage::ProbeAnswer {
            from: 1,
            relay_for: None,
        },
        count,
    );
    cluster.drop_message(
        1,
        2,
        TransportMessage::ProbeAnswer {
            from: 1,
            relay_for: Some(0),
        },
        count,
    );
    cluster
}

fn concluded(size: usize, required: usize, threshold: usize) -> Cluster {
    let mut cluster = Cluster::new(size, required, threshold);
    cluster.start_all();
    cluster.run(ROUNDS);
    cluster
}

// Node 0 hears from node 1, as an answer reaching it says.
fn heard_from_one(cluster: Cluster) -> Cluster {
    let mut cluster = cluster;
    cluster.inject_transport(
        0,
        TransportMessage::ProbeAnswer {
            from: 1,
            relay_for: None,
        },
    );
    cluster.run(ROUNDS);
    cluster
}

// A whole period of node 0's in which it probes node 1 and node 1 misses it,
// directly and through the helper.
fn one_missed_period(cluster: Cluster) -> Cluster {
    let mut cluster = probed(cluster, 1);
    cluster.elapse(0, TimerMessage::ProbeMissed { peer_id: 1 });
    cluster.run(ROUNDS);
    cluster.elapse(0, TimerMessage::IndirectProbesMissed { peer_id: 1 });
    cluster.run(ROUNDS);
    cluster
}

// Node 0's periods passed until the one in which it probes `peer`.
fn probed(cluster: Cluster, peer: u64) -> Cluster {
    let mut cluster = cluster;
    for _ in 0..4 {
        cluster.elapse(0, TimerMessage::ProbeDue);
        cluster.run(ROUNDS);
        if cluster.last_probed(0) == Some(peer) {
            break;
        }
    }
    cluster
}

#[test]
fn a_lowered_count_restores_the_quorum() {
    // Arrange
    let mut cluster = concluded(3, 2, 1);
    cluster.change_required_count(0, 4);

    // Act
    cluster.change_required_count(0, 2);

    // Assert
    assert_eq!(cluster.quorum(0), Quorum::Held);
}

#[test]
fn a_member_added_mid_round_is_probed_in_the_same_round() {
    // Arrange
    let mut cluster = concluded(3, 2, 1);
    cluster.add_newcomer(NEWCOMER);
    cluster = probed(cluster, 1);
    cluster.request_addition(0, NEWCOMER);
    cluster.run(ROUNDS);
    for index in 0..4 {
        cluster.commit_addition(index, NEWCOMER);
    }
    cluster.run(ROUNDS);

    // Act
    let probes: Vec<Option<u64>> = (0..3)
        .map(|_| {
            cluster.elapse(0, TimerMessage::ProbeDue);
            cluster.run(ROUNDS);
            cluster.last_probed(0)
        })
        .collect();

    // Assert
    assert_eq!(probes, vec![Some(2), Some(NEWCOMER), Some(1)]);
}

#[test]
fn a_member_missing_as_many_periods_running_as_the_threshold_is_suspected() {
    // Arrange
    let cluster = answers_of_one_lost(concluded(3, 2, 3), 3);
    let cluster = one_missed_period(one_missed_period(cluster));

    // Act
    let mut cluster = one_missed_period(cluster);

    // Assert
    assert_eq!(cluster.suspected(0), vec![1]);
}

#[test]
fn a_member_missing_fewer_periods_running_than_the_threshold_is_never_suspected() {
    // Arrange
    let cluster = answers_of_one_lost(concluded(3, 2, 3), 4);
    let cluster = heard_from_one(one_missed_period(one_missed_period(cluster)));

    // Act
    let mut cluster = one_missed_period(one_missed_period(cluster));

    // Assert
    assert!(cluster.suspected(0).is_empty());
}

#[test]
fn a_member_whose_answers_are_all_lost_is_suspected() {
    // Arrange
    let cluster = answers_of_one_lost(concluded(3, 2, 1), 1);

    // Act
    let mut cluster = one_missed_period(cluster);

    // Assert
    assert_eq!(cluster.suspected(0), vec![1]);
}

#[test]
fn a_node_still_bootstrapping_holds_no_probe_period() {
    // Arrange
    let mut cluster = Cluster::new(3, 3, 1);
    cluster.start_all();

    // Act
    let elapsed = cluster.elapse(0, TimerMessage::ProbeDue);

    // Assert
    assert!(!elapsed);
    assert!(!cluster.is_held(0, &TimerMessage::ProbeDue));
}

#[test]
fn a_raised_count_loses_the_quorum() {
    // Arrange
    let mut cluster = concluded(3, 2, 1);

    // Act
    cluster.change_required_count(0, 4);

    // Assert
    assert_eq!(cluster.quorum(0), Quorum::Lost);
}

#[test]
fn a_suspect_heard_from_again_is_revived() {
    // Arrange
    let cluster = one_missed_period(answers_of_one_lost(concluded(3, 2, 1), 1));

    // Act
    let mut cluster = probed(cluster, 1);

    // Assert
    assert!(cluster.suspected(0).is_empty());
}

#[test]
fn an_answer_lost_on_the_direct_path_is_saved_by_an_indirect_one() {
    // Arrange
    let mut cluster = concluded(3, 2, 1);
    cluster.drop_message(
        1,
        0,
        TransportMessage::ProbeAnswer {
            from: 1,
            relay_for: None,
        },
        1,
    );
    cluster = probed(cluster, 1);

    // Act
    cluster.elapse(0, TimerMessage::ProbeMissed { peer_id: 1 });
    cluster.run(ROUNDS);

    // Assert
    assert!(!cluster.is_held(0, &TimerMessage::IndirectProbesMissed { peer_id: 1 }));
    assert!(cluster.suspected(0).is_empty());
}

#[test]
fn every_other_member_is_probed_within_one_round_of_periods() {
    // Arrange
    let mut cluster = concluded(4, 2, 1);

    // Act
    let probes: Vec<Option<u64>> = (0..3)
        .map(|_| {
            cluster.elapse(0, TimerMessage::ProbeDue);
            cluster.run(ROUNDS);
            cluster.last_probed(0)
        })
        .collect();

    // Assert
    assert_eq!(probes, vec![Some(1), Some(2), Some(3)]);
}
