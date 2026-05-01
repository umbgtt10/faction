// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod common;

use faction_protocol::transport_message::TransportMessage;

use common::cluster::Cluster;

#[test]
fn five_nodes_converge_to_bootstrapped() {
    let mut cluster = Cluster::new(5, 4);
    cluster.converge();
    assert!(cluster.is_bootstrapped());
}

#[test]
fn cluster_fails_when_ready_message_is_dropped() {
    let mut cluster = Cluster::new(2, 2);
    cluster.drop_message(0, 1, TransportMessage::Ready { from: 0 }, 1);
    cluster.converge();
    assert!(!cluster.is_bootstrapped());
}
