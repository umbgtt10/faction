// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::fake_time;
use crate::support::real_time;
use crate::support::recipe;
use crate::traits::monotonic_clock_tests::FakeMonotonicClock;
use faction::addition::Addition;
use faction::peer_state::PeerState;
use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;
use faction_system_tests::infra::transport::in_memory::in_memory_transport::InMemoryTransport;
use faction_system_tests::node::Node;
use faction_system_tests::spawn::Spawn;
use faction_system_tests::timer_delay::TimerDelay;
use faction_system_tests::traits::monotonic_clock::MonotonicClock;
use std::sync::Arc;
use std::time::Duration;

fn cluster() -> (Cluster, Arc<FakeMonotonicClock>) {
    let (time, clock) = fake_time();
    let recipe = recipe(2, &time);
    let nodes: Vec<Node> = InMemoryTransport::new_mesh(&[0, 1])
        .into_iter()
        .zip([0, 1])
        .map(|(transport, id)| recipe.node(Spawn::Task, id, vec![0, 1], Box::new(transport)))
        .collect();
    let cluster = Cluster::new(
        nodes,
        Spawn::Task,
        TimerDelay::Minimal,
        None,
        None,
        time.pause.clone(),
    );
    (cluster, clock)
}

#[test]
fn abort_addition_of_a_requested_addition_withdraws_it() {
    // Arrange
    let mut cluster = ClusterBuilder::new(3, 2, real_time()).build();
    cluster.poll_until_bootstrapped();
    cluster.add_newcomer(3);
    cluster.request_addition(1, 3);
    let requested = cluster.snapshot(1).addition;

    // Act
    cluster.abort_addition(1, 3);

    // Assert
    assert_eq!(requested, Addition::Requested { peer_id: 3 });
    assert_eq!(cluster.snapshot(1).addition, Addition::None);
}

#[test]
fn new_holds_the_nodes_it_is_given() {
    // Arrange & Act
    let (cluster, _) = cluster();

    // Assert
    assert_eq!(cluster.node_count(), 2);
    assert_eq!(cluster.node_state(0), PeerState::Fresh);
}

#[test]
fn settle_pauses_once_a_round() {
    // Arrange
    let (mut cluster, clock) = cluster();

    // Act
    cluster.settle(3);

    // Assert
    assert_eq!(clock.now(), Duration::from_millis(30));
}

#[test]
fn step_all_steps_every_node() {
    // Arrange
    let (mut cluster, clock) = cluster();
    cluster.start_all();
    clock.advance(Duration::from_millis(10));

    // Act
    cluster.step_all();

    // Assert
    assert_eq!(cluster.node_state(0), PeerState::Pinging);
    assert_eq!(cluster.node_state(1), PeerState::Pinging);
}
