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

    // Act
    cluster.step_timer();
    cluster.step_timer();
    cluster.step_timer();

    cluster.step_transport();

    // Assert
    assert!(cluster.is_bootstrapped());
}
