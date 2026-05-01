// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_system_tests::cluster::Cluster;

#[test]
fn pre_accumulated_ready_counts_toward_quorum_when_local_completion_fires() {
    // Arrange
    let mut cluster = Cluster::new(5, 4);
    cluster.start_node(0);
    cluster.start_node(1);

    // Act — node 0 processes all ParticipationObserved timers then LPC
    for _ in 0..4 {
        cluster.step_timer_node(0);
    }
    cluster.step_timer_node(0);

    // Node 1 drains Pings from start, then accumulates a Ready from node 0
    for _ in 0..4 {
        cluster.step_transport_node(1);
    }
    cluster.step_transport_node(1);

    // Assert — Ready is pre-accumulated in Pinging state
    assert_eq!(cluster.node_collecting_peers_len(1), 1);
}
