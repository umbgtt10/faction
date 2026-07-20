// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::timer_message::TimerMessage;

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
fn cluster_recovers_after_deadline_via_late_readiness(#[case] size: usize, #[case] quorum: usize) {
    // Arrange
    let mut cluster = Cluster::new(size, quorum);
    cluster.start_all();
    for i in 0..size {
        cluster.inject_timer(i, TimerMessage::DeadlineExpired);
    }

    // Act — fire every node's deadline before any readiness is exchanged
    let steps = 2 * size + 5;
    for i in 0..size {
        for _ in 0..steps {
            cluster.step_timer_node(i);
        }
    }
    for i in 0..size {
        assert!(
            cluster.is_timed_out(i),
            "size {size}, quorum {quorum}: node {i} did not report TimedOut before recovery"
        );
    }

    // Act — the cluster keeps running; late readiness still drives it to quorum
    let rounds = 10 * size + 50;
    for _ in 0..rounds {
        for i in 0..size {
            cluster.step_timer_node(i);
            cluster.step_transport_node(i);
        }
    }

    // Assert — a missed deadline is not terminal; the cluster recovers
    assert!(
        cluster.is_bootstrapped(),
        "size {size}, quorum {quorum}: cluster did not recover after the deadline"
    );
}
