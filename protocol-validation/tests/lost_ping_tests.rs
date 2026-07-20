// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;

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
fn cluster_converges_when_a_ping_is_dropped(#[case] size: usize, #[case] quorum: usize) {
    // Arrange
    let mut cluster = Cluster::new(size, quorum);
    cluster.drop_message(0, 1, TransportMessage::Ping { from: 0 }, 1);
    cluster.start_all();

    // Act
    let rounds = 10 * size + 50;
    for _ in 0..rounds {
        for i in 0..size {
            cluster.step_timer_node(i);
            cluster.step_transport_node(i);
        }
    }

    // Assert
    assert!(
        cluster.is_bootstrapped(),
        "size {size}, quorum {quorum}: cluster did not reach Bootstrapped after a dropped ping"
    );
}
