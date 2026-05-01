// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_system_tests::cluster::Cluster;

#[test]
fn five_nodes_converge_to_bootstrapped() {
    // Arrange
    let mut cluster = Cluster::new(5, 4);
    cluster.start_all();

    // Act — 5 timer phases: each node processes 4 ParticipationObserved + 1 LocalParticipationCompleted
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);

    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);

    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);

    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);

    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);

    // Act — 3 transport phases: each node processes 3 Ready messages → quorum
    cluster.step_transport_node(0);
    cluster.step_transport_node(1);
    cluster.step_transport_node(2);
    cluster.step_transport_node(3);
    cluster.step_transport_node(4);

    cluster.step_transport_node(0);
    cluster.step_transport_node(1);
    cluster.step_transport_node(2);
    cluster.step_transport_node(3);
    cluster.step_transport_node(4);

    cluster.step_transport_node(0);
    cluster.step_transport_node(1);
    cluster.step_transport_node(2);
    cluster.step_transport_node(3);
    cluster.step_transport_node(4);

    // Assert
    assert!(cluster.is_bootstrapped());
}
