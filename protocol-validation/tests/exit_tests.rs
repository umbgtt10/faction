// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

use faction_protocol_validation::cluster::Cluster;
use rstest::rstest;

fn converge(cluster: &mut Cluster, size: usize) {
    cluster.start_all();
    for _ in 0..(10 * size + 50) {
        for i in 0..size {
            cluster.step_timer_node(i);
            cluster.step_transport_node(i);
        }
    }
}

#[rstest]
#[case(3, 2)]
#[case(4, 3)]
#[case(5, 3)]
#[case(5, 4)]
#[case(6, 4)]
#[case(7, 4)]
#[case(7, 5)]
#[case(8, 5)]
#[case(9, 5)]
#[case(9, 6)]
#[case(10, 6)]
#[case(10, 7)]
#[case(11, 6)]
#[case(12, 7)]
#[case(13, 9)]
#[case(3, 3)]
#[case(4, 4)]
#[case(5, 5)]
#[case(6, 6)]
#[case(8, 8)]
#[case(2, 2)]
fn bootstrapped_node_stays_silent_to_further_signals(#[case] size: usize, #[case] quorum: usize) {
    // Arrange
    let mut cluster = Cluster::new(size, quorum);
    converge(&mut cluster, size);
    assert!(
        cluster.is_bootstrapped(),
        "size {size}, quorum {quorum}: did not converge"
    );
    for i in 0..size {
        while cluster.step_transport_node(i) {}
        while cluster.step_timer_node(i) {}
    }

    // Act & Assert — a self RetryReady on a bootstrapped node re-broadcasts nothing
    cluster.inject_timer(0, TimerMessage::RetryReady);
    cluster.step_timer_node(0);
    assert!(
        !cluster.step_transport_node(1),
        "size {size}: RetryReady re-broadcast to a peer"
    );

    // Act & Assert — a peer's ready draws no reply and the node stays bootstrapped
    cluster.inject_transport(0, TransportMessage::Ready { from: 1 });
    cluster.step_transport_node(0);
    assert!(
        !cluster.step_transport_node(1),
        "size {size}: bootstrapped node replied to a peer ready"
    );
    assert!(cluster.is_bootstrapped());
}
