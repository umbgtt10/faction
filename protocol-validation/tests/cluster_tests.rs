// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;
use faction_protocol_validation::cluster::Cluster;

#[test]
fn drop_message_withholds_the_matching_message_count_times() {
    // Arrange
    let mut cluster = Cluster::new(2, 2);
    cluster.drop_message(0, 1, TransportMessage::Ready { from: 0 }, 1);
    cluster.start_node(0);

    // Act
    cluster.step_timer_node(0);
    cluster.step_timer_node(0);
    for _ in 0..4 {
        cluster.step_transport_node(1);
    }

    // Assert
    assert_eq!(cluster.node_collecting_peers_len(1), 0);

    // Act
    cluster.step_timer_node(0);
    cluster.step_timer_node(0);
    for _ in 0..4 {
        cluster.step_transport_node(1);
    }

    // Assert
    assert_eq!(cluster.node_collecting_peers_len(1), 1);
}

#[test]
fn inject_transport_ready_is_counted_by_the_node() {
    // Arrange
    let mut cluster = Cluster::new(3, 2);

    // Act
    cluster.inject_transport(1, TransportMessage::Ready { from: 0 });
    cluster.step_transport_node(1);

    // Assert
    assert_eq!(cluster.node_collecting_peers_len(1), 1);
}

#[test]
fn is_bootstrapped_before_start_returns_false() {
    // Arrange
    let mut cluster = Cluster::new(3, 2);

    // Act
    let is_bootstrapped = cluster.is_bootstrapped();

    // Assert
    assert!(!is_bootstrapped);
}

#[test]
fn is_timed_out_fresh_node_returns_false() {
    // Arrange
    let mut cluster = Cluster::new(3, 2);

    // Act
    let is_timed_out = cluster.is_timed_out(0);

    // Assert
    assert!(!is_timed_out);
}

#[test]
fn node_collecting_peers_len_fresh_node_returns_zero() {
    // Arrange
    let mut cluster = Cluster::new(3, 2);

    // Act
    let collecting = cluster.node_collecting_peers_len(2);

    // Assert
    assert_eq!(collecting, 0);
}
