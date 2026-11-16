// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// Epochs and rejoin in the deterministic cluster: the test plays the layer
// that commits, node by node, and node 0 coordinates. The system tests'
// scenarios, in process: a node restarted from genesis catches up, one
// restarted from a snapshot starts at its epoch, a removed node invited back
// joins as a newcomer, a node the layer above has not reached is heard behind
// and never suspected for it, and a node ahead refuses readiness from behind
// until its peers adopt its membership.

use faction::peer_state::PeerState;
use faction::reconfiguration::Reconfiguration;
use faction_protocol::commit::Commit;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol_validation::cluster::Cluster;

const NEWCOMER: u64 = 3;
const ROUNDS: usize = 100;

fn concluded(size: usize, required: usize) -> Cluster {
    let mut cluster = Cluster::new(size, required, 1);
    cluster.start_all();
    cluster.run(ROUNDS);
    cluster
}

// Three nodes requiring two, concluded, then the newcomer added at epoch 1.
fn grown() -> Cluster {
    let mut cluster = concluded(3, 2);
    cluster.add_newcomer(NEWCOMER);
    cluster.request_addition(0, NEWCOMER);
    cluster.run(ROUNDS);
    for index in 0..4 {
        cluster.commit(
            index,
            Commit::Addition {
                peer_id: NEWCOMER,
                epoch: 1,
            },
        );
    }
    cluster.run(ROUNDS);
    cluster
}

#[test]
fn a_node_ahead_refuses_readiness_from_behind_until_its_peers_adopt_its_membership() {
    // Arrange
    let mut cluster = Cluster::new(3, 3, 1);
    cluster.restart(0, vec![0, 1, 2], 3, 1);

    // Act
    cluster.start_all();
    cluster.run(ROUNDS);

    // Assert
    assert!(cluster.is_bootstrapped());
    assert!((0..3).all(|index| cluster.epoch(index) == 1));
}

#[test]
fn a_node_behind_is_heard_never_suspected_and_caught_up_by_the_layer_above() {
    // Arrange
    let mut cluster = concluded(4, 2);
    let removal = Commit::Removal {
        peer_id: 2,
        epoch: 1,
    };
    for index in 0..3 {
        cluster.commit(index, removal);
    }
    cluster.run(ROUNDS);

    // Act
    for _ in 0..4 {
        cluster.elapse(0, TimerMessage::ProbeDue);
        cluster.run(ROUNDS);
    }
    let behind = cluster.epoch(3);
    cluster.commit(3, removal);
    cluster.run(ROUNDS);

    // Assert
    assert_eq!(behind, 0);
    assert!(cluster.suspected(0).is_empty());
    assert_eq!(cluster.epoch(3), 1);
    assert_eq!(cluster.members(3), vec![0, 1, 3]);
}

#[test]
fn a_node_restarted_from_a_snapshot_starts_at_its_epoch() {
    // Arrange
    let mut cluster = grown();

    // Act
    cluster.restart(1, vec![0, 1, 2, NEWCOMER], 2, 1);
    cluster.start_node(1);
    cluster.run(ROUNDS);

    // Assert
    assert_eq!(cluster.node_state(1), PeerState::Bootstrapped);
    assert_eq!((cluster.members(1).len(), cluster.epoch(1)), (4, 1));
}

#[test]
fn a_node_restarted_from_genesis_catches_up_with_an_addition() {
    // Arrange
    let mut cluster = grown();

    // Act
    cluster.restart(1, vec![0, 1, 2], 2, 0);
    cluster.start_node(1);
    cluster.run(ROUNDS);

    // Assert
    assert_eq!(cluster.node_state(1), PeerState::Bootstrapped);
    assert_eq!((cluster.members(1).len(), cluster.epoch(1)), (4, 1));
}

#[test]
fn a_removed_node_invited_back_joins_as_a_newcomer() {
    // Arrange
    let mut cluster = concluded(4, 2);
    for index in 0..4 {
        cluster.commit(
            index,
            Commit::Removal {
                peer_id: 3,
                epoch: 1,
            },
        );
    }
    cluster.run(ROUNDS);
    let removed = cluster.node_state(3);

    // Act
    cluster.request_addition(0, 3);
    cluster.run(ROUNDS);
    let acknowledged = cluster.reconfiguration(0);
    for index in 0..4 {
        cluster.commit(
            index,
            Commit::Addition {
                peer_id: 3,
                epoch: 2,
            },
        );
    }
    cluster.run(ROUNDS);

    // Assert
    assert_eq!(removed, PeerState::Removed);
    assert_eq!(
        acknowledged,
        Reconfiguration::AdditionAcknowledged { peer_id: 3 }
    );
    assert!(cluster.is_bootstrapped());
    assert!((0..4).all(|index| cluster.members(index).len() == 4 && cluster.epoch(index) == 2));
}
