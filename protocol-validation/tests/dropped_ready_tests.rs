// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::transport_message::TransportMessage;

use faction_protocol_validation::cluster::Cluster;

#[test]
fn cluster_converges_via_retry_when_ready_message_is_dropped() {
    // Arrange
    let mut cluster = Cluster::new(5, 4);
    cluster.drop_message(0, 1, TransportMessage::Ready { from: 0 }, 1);
    cluster.start_all();

    // Act — drain initial Pings from start_all (4 per node)
    for _ in 0..4 {
        cluster.step_transport_node(0);
        cluster.step_transport_node(1);
        cluster.step_transport_node(2);
        cluster.step_transport_node(3);
        cluster.step_transport_node(4);
    }

    // Act — timer phases 1-4: ParticipationObserved for all remote peers
    for _ in 0..4 {
        cluster.step_timer_node(0);
        cluster.step_timer_node(1);
        cluster.step_timer_node(2);
        cluster.step_timer_node(3);
        cluster.step_timer_node(4);
    }

    // Act — timer phase 5: LocalParticipationCompleted
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);

    // Act — timer phase 6: RetryPing fires
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);

    // Act — timer phase 7: RetryReady fires (node 0 re-sends Ready, not dropped now)
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);

    // Act — transport phases: drain extra Pings then process Readys → quorum
    for _ in 0..3 {
        cluster.step_transport_node(0);
        cluster.step_transport_node(1);
        cluster.step_transport_node(2);
        cluster.step_transport_node(3);
        cluster.step_transport_node(4);
    }

    // Assert
    assert!(cluster.is_bootstrapped());
}
