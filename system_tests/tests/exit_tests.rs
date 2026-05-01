// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

use faction_system_tests::cluster::Cluster;

fn converge_two_nodes(cluster: &mut Cluster) {
    cluster.start_all();

    cluster.step_timer_node(0);
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(1);
    cluster.step_timer_node(1);

    cluster.step_transport_node(0);
    cluster.step_transport_node(0);
    cluster.step_transport_node(1);
}

#[test]
fn retry_ready_on_bootstrapped_node_produces_noop_and_no_transport_messages() {
    // Arrange — converge two nodes
    let mut cluster = Cluster::new(2, 2);
    converge_two_nodes(&mut cluster);
    assert!(cluster.is_bootstrapped());

    // Drain any pending transport messages
    cluster.step_transport_node(0);
    cluster.step_transport_node(1);

    // Act — inject RetryReady into bootstrapped node 0
    cluster.inject_timer(0, TimerMessage::RetryReady);
    cluster.step_timer_node(0);

    // Assert — no transport messages were produced (node 1 inbox is empty)
    assert!(!cluster.step_transport_node(1));
}

#[test]
fn pending_timers_are_cancelled_on_bootstrapped() {
    // Arrange — converge two nodes
    let mut cluster = Cluster::new(2, 2);
    converge_two_nodes(&mut cluster);
    assert!(cluster.is_bootstrapped());

    // Drain any pending transport messages
    cluster.step_transport_node(0);
    cluster.step_transport_node(1);

    // Assert — no pending timer events on either node
    assert!(!cluster.step_timer_node(0));
    assert!(!cluster.step_timer_node(1));
}

#[test]
fn duplicate_ready_after_bootstrapped_produces_noop() {
    // Arrange — converge two nodes
    let mut cluster = Cluster::new(2, 2);
    converge_two_nodes(&mut cluster);
    assert!(cluster.is_bootstrapped());

    // Drain any pending transport messages
    cluster.step_transport_node(0);
    cluster.step_transport_node(1);

    // Act — inject a duplicate Ready into node 0's transport inbox
    cluster.inject_transport(0, 1, TransportMessage::Ready { from: 1 });
    cluster.step_transport_node(0);

    // Assert — Noop output, no transport messages produced (node 1 inbox still empty)
    assert!(!cluster.step_transport_node(1));
    // Node 0 remains Bootstrapped
    assert!(cluster.is_bootstrapped());
}
