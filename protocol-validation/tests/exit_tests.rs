// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

use faction_protocol_validation::cluster::Cluster;

fn converge(cluster: &mut Cluster) {
    cluster.start_all();

    for _ in 0..4 {
        cluster.step_timer_node(0);
        cluster.step_timer_node(1);
        cluster.step_timer_node(2);
        cluster.step_timer_node(3);
        cluster.step_timer_node(4);
    }
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);
    cluster.step_timer_node(0);
    cluster.step_timer_node(1);
    cluster.step_timer_node(2);
    cluster.step_timer_node(3);
    cluster.step_timer_node(4);

    for _ in 0..12 {
        cluster.step_transport_node(0);
        cluster.step_transport_node(1);
        cluster.step_transport_node(2);
        cluster.step_transport_node(3);
        cluster.step_transport_node(4);
    }
}

#[test]
fn retry_ready_on_bootstrapped_node_produces_noop_and_no_transport_messages() {
    let mut cluster = Cluster::new(5, 4);
    converge(&mut cluster);
    assert!(cluster.is_bootstrapped());

    while cluster.step_transport_node(0) {}
    while cluster.step_transport_node(1) {}

    cluster.inject_timer(0, TimerMessage::RetryReady);
    cluster.step_timer_node(0);

    assert!(!cluster.step_transport_node(1));
}

#[test]
fn pending_timers_are_cancelled_on_bootstrapped() {
    let mut cluster = Cluster::new(5, 4);
    converge(&mut cluster);
    assert!(cluster.is_bootstrapped());

    while cluster.step_transport_node(0) {}
    while cluster.step_transport_node(1) {}
    while cluster.step_timer_node(0) {}
    while cluster.step_timer_node(1) {}

    assert!(!cluster.step_timer_node(0));
    assert!(!cluster.step_timer_node(1));
}

#[test]
fn duplicate_ready_after_bootstrapped_produces_noop() {
    let mut cluster = Cluster::new(5, 4);
    converge(&mut cluster);
    assert!(cluster.is_bootstrapped());

    while cluster.step_transport_node(0) {}
    while cluster.step_transport_node(1) {}

    cluster.inject_transport(0, 1, TransportMessage::Ready { from: 1 });
    cluster.step_transport_node(0);

    assert!(!cluster.step_transport_node(1));
    assert!(cluster.is_bootstrapped());
}
