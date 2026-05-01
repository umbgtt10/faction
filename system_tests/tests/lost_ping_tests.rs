// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::transport_message::TransportMessage;

use faction_system_tests::cluster::Cluster;

#[test]
fn cluster_converges_via_retry_when_ping_message_is_dropped() {
    // Arrange
    let mut cluster = Cluster::new(2, 2);
    cluster.drop_message(0, 1, TransportMessage::Ping { from: 0 }, 1);
    cluster.start_all();

    // Act — drain initial Ping from node 1 to node 0 (node 0→1 Ping was dropped)
    cluster.step_transport_node(0);

    // Act — timer phase 1: both nodes process ParticipationObserved
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);

    // Act — timer phase 2: both nodes process LocalParticipationCompleted
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);

    // Act — timer phase 3: RetryPing fires on both nodes
    // Node 0 re-sends Ping to node 1 (not dropped this time)
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);

    // Node 1 receives the retried Ping
    cluster.step_transport_node(1);

    // Act — timer phase 4: RetryReady fires on both nodes
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);

    // Act — Ready exchange and convergence
    cluster.step_transport_node(0);
    cluster.step_transport_node(0);
    cluster.step_transport_node(1);

    // Assert
    assert!(cluster.is_bootstrapped());
}
