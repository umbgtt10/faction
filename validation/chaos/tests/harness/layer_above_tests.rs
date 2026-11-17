// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction_chaos::harness::layer_above::LayerAbove;
use faction_chaos::model::change::Change;
use faction_chaos::model::planned_change::PlannedChange;
use faction_chaos::model::scenario::Scenario;
use faction_protocol::commit::Commit;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::infra::timer::simulated::simulated_clock::SimulatedClock;
use faction_system_tests::time_ports::TimePorts;
use std::sync::Arc;
use std::time::Duration;

// A cluster of three, two required, with a newcomer 3 waiting, on a
// simulated clock.
fn cluster() -> (Cluster, Arc<SimulatedClock>) {
    let clock = Arc::new(SimulatedClock::at(Duration::ZERO));
    let time = TimePorts {
        clock: clock.clone(),
        pause: clock.clone(),
        wall_clock: Arc::new(SystemWallClock),
    };
    let mut cluster = ClusterBuilder::new(3, 2, time).build();
    cluster.start_all();
    cluster.add_newcomer(3);
    (cluster, clock)
}

fn planned(change: Change, after: Option<usize>) -> PlannedChange {
    PlannedChange {
        change,
        due: 0,
        after,
    }
}

// `rounds` rounds of the layer above, every node stepping as far as it has
// inputs after each.
fn play(layer: &mut LayerAbove, cluster: &Cluster, clock: &SimulatedClock, rounds: u64) {
    for round in 0..rounds {
        layer.act(round, cluster);
        for index in 0..cluster.node_count() {
            for _ in 0..16 {
                cluster.step(index);
            }
        }
        clock.advance(Duration::from_millis(10));
    }
}

fn scenario(changes: Vec<PlannedChange>, delivery_lag: u64) -> Scenario {
    Scenario {
        seed: 3,
        nodes: 3,
        required_count: 2,
        suspicion_threshold: 1,
        changes,
        faults: Vec::new(),
        restarts: Vec::new(),
        delivery_lag,
        duplicates: 0,
        chaos_rounds: 100,
        quiescence_rounds: 100,
    }
}

#[test]
fn act_commits_a_new_required_count_of_its_own_accord() {
    // Arrange
    let (cluster, clock) = cluster();
    let mut layer = LayerAbove::new(&scenario(
        vec![planned(Change::RequiredCount { required_count: 3 }, None)],
        0,
    ));

    // Act
    play(&mut layer, &cluster, &clock, 50);

    // Assert
    assert_eq!(
        layer.log().commits(),
        [Commit::RequiredCount {
            required_count: 3,
            epoch: 1
        }]
    );
    assert!(layer.settled());
    assert_eq!(cluster.snapshot(2).required_count, 3);
}

#[test]
fn act_commits_an_addition_once_its_newcomer_acknowledges() {
    // Arrange
    let (cluster, clock) = cluster();
    let mut layer = LayerAbove::new(&scenario(
        vec![planned(Change::Addition { peer_id: 3 }, None)],
        0,
    ));

    // Act
    play(&mut layer, &cluster, &clock, 200);

    // Assert
    assert_eq!(
        layer.log().commits(),
        [Commit::Addition {
            peer_id: 3,
            epoch: 1
        }]
    );
    assert!(layer.settled());
    assert_eq!(cluster.snapshot(3).members, vec![0, 1, 2, 3]);
    assert_eq!(cluster.snapshot(0).epoch, 1);
}

#[test]
fn act_takes_a_removed_member_back_only_after_its_removal() {
    // Arrange
    let (cluster, clock) = cluster();
    let mut layer = LayerAbove::new(&scenario(
        vec![
            planned(Change::Addition { peer_id: 2 }, Some(1)),
            planned(Change::Removal { peer_id: 2 }, None),
        ],
        0,
    ));

    // Act
    play(&mut layer, &cluster, &clock, 300);

    // Assert
    assert_eq!(
        layer.log().commits(),
        [
            Commit::Removal {
                peer_id: 2,
                epoch: 1
            },
            Commit::Addition {
                peer_id: 2,
                epoch: 2
            },
        ]
    );
    assert_eq!(cluster.node_state(2), PeerState::Bootstrapped);
    assert_eq!(cluster.snapshot(2).epoch, 2);
}

#[test]
fn log_of_a_layer_yet_to_act_is_empty() {
    // Arrange & Act
    let layer = LayerAbove::new(&scenario(Vec::new(), 0));

    // Assert
    assert!(layer.log().commits().is_empty());
    assert!(layer.settled());
}

#[test]
fn quiesce_reports_every_commit_late_no_more() {
    // Arrange
    let (cluster, clock) = cluster();
    let mut layer = LayerAbove::new(&scenario(
        vec![planned(Change::RequiredCount { required_count: 3 }, None)],
        5_000,
    ));
    play(&mut layer, &cluster, &clock, 10);
    let late = cluster.snapshot(1).epoch;

    // Act
    layer.quiesce();
    play(&mut layer, &cluster, &clock, 1);

    // Assert
    assert_eq!(layer.log().commits().len(), 1);
    assert_eq!(late, 0);
    assert_eq!(cluster.snapshot(1).epoch, 1);
}

#[test]
fn settled_with_a_change_not_yet_committed_is_false() {
    // Arrange & Act
    let layer = LayerAbove::new(&scenario(
        vec![planned(Change::Removal { peer_id: 1 }, None)],
        0,
    ));

    // Assert
    assert!(!layer.settled());
}
