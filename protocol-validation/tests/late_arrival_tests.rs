// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol_validation::cluster::Cluster;
use rstest::rstest;

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
#[case(2, 1)]
#[case(2, 2)]
#[case(5, 1)]
fn pre_accumulated_ready_counts_toward_quorum_when_local_completion_fires(
    #[case] size: usize,
    #[case] quorum: usize,
) {
    // Arrange
    let mut cluster = Cluster::new(size, quorum);
    cluster.start_node(0);
    cluster.start_node(1);

    // Act — node 0 completes participation, then broadcasts its readiness
    for _ in 0..(size - 1) {
        cluster.step_timer_node(0);
    }
    cluster.step_timer_node(0);

    // Node 1 drains its inbox, pre-accumulating node 0's ready while still pinging
    for _ in 0..size {
        cluster.step_transport_node(1);
    }

    // Assert — the ready is pre-accumulated in the Pinging state
    assert_eq!(
        cluster.node_collecting_peers_len(1),
        1,
        "size {size}, quorum {quorum}: node 1 did not pre-accumulate node 0's ready"
    );
}
