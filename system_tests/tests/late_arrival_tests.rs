// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_system_tests::cluster::Cluster;

#[test]
fn pre_accumulated_ready_counts_toward_quorum_when_local_completion_fires() {
    // Arrange
    let mut cluster = Cluster::new(2, 2);
    cluster.start_node(0);
    cluster.start_node(1);

    // Act — node 0 races ahead: ParticipationObserved then LocalParticipationCompleted
    cluster.step_timer_node(0);
    cluster.step_timer_node(0);

    // Node 1 receives Ready from node 0 before its own LocalParticipationCompleted
    cluster.step_transport_node(1);

    // Assert — Ready is pre-accumulated in Pinging state
    assert_eq!(cluster.node_collecting_peers_len(1), 1);

    // Act — node 1 now processes its own timers
    cluster.step_timer_node(1);
    cluster.step_timer_node(1);

    // Node 0 receives Ready back from node 1
    cluster.step_transport_node(0);

    // Assert
    assert!(cluster.is_bootstrapped());
}
