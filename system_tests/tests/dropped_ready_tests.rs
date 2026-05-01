// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::transport_message::TransportMessage;

use faction_system_tests::cluster::Cluster;

#[test]
fn cluster_converges_via_retry_when_ready_message_is_dropped() {
    // Arrange
    let mut cluster = Cluster::new(2, 2);
    cluster.drop_message(0, 1, TransportMessage::Ready { from: 0 }, 1);
    cluster.start_all();

    // Act — timer phase 1: each node processes ParticipationObserved
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);

    // Act — timer phase 2: each node processes LocalParticipationCompleted
    // Node 0→1 Ready is dropped; both schedule RetryReady
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);

    // Act — timer phase 3: RetryReady fires on both nodes
    // Node 0 re-sends Ready (not dropped this time, counter exhausted)
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);

    // Act — transport phase: both nodes process the Ready → quorum
    cluster.step_transport_node(0);
    cluster.step_transport_node(1);

    // Assert
    assert!(cluster.is_bootstrapped());
}
