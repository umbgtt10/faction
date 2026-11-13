// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The replay tier, a cluster: every node of a simulated cluster logs its calls
// as the specification writes them, and every node's log replays whole into a
// node created from that node's configuration -- however the cluster's
// signals, completions and deadlines interleaved.

use crate::support::log_replay::LogReplay;
use crate::support::spec_log::Lines;
use crate::support::spec_log::SpecLog;
use faction::observer::Observer;
use faction::types::PeerId;
use faction_core_validation::cluster_simulation::ClusterSimulation;
use proptest::prelude::*;
use serde_json::json;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

// Every node's log, its configuration first, replayed: the departures found.
fn departures(lines: &[Lines], required_count: usize) -> Vec<String> {
    let peers: Vec<Value> = (0..lines.len())
        .map(|peer| json!(peer.to_string()))
        .collect();
    lines
        .iter()
        .enumerate()
        .flat_map(|(peer, lines)| {
            let configuration = json!({
                "peer_id": peer.to_string(),
                "peers": peers,
                "required_count": required_count.to_string(),
                "suspicion_threshold": "1",
            });
            let log = [vec![configuration], lines.borrow().clone()].concat();
            LogReplay::replay(&log)
                .departures
                .into_iter()
                .map(move |departure| format!("node {peer}, {departure}"))
        })
        .collect()
}

// A simulated cluster of `size` nodes, each logging to its own lines.
fn logging_cluster(size: usize, required_count: usize) -> (ClusterSimulation, Vec<Lines>) {
    let lines: Vec<Lines> = (0..size)
        .map(|_| Rc::new(RefCell::new(Vec::new())))
        .collect();
    let observers = lines
        .iter()
        .map(|lines| Box::new(SpecLog::new(Rc::clone(lines))) as Box<dyn Observer>)
        .collect();
    (ClusterSimulation::new(observers, required_count), lines)
}

proptest! {
    #[test]
    fn replay_of_every_node_s_log_reconstructs_it_however_the_cluster_ran(
        required_count in 1usize..=5,
        actions in prop::collection::vec((0u8..4, 0u64..5), 0..30),
    ) {
        // Arrange
        let (mut cluster, lines) = logging_cluster(5, required_count);
        for (action, peer) in actions {
            match action {
                0 => cluster.inject_participation(peer),
                1 => cluster.inject_ready(peer),
                2 => cluster.complete_local(peer),
                _ => cluster.expire_deadline(peer),
            }
        }

        // Act
        let departures = departures(&lines, required_count);

        // Assert
        prop_assert!(departures.is_empty(), "{departures:?}");
    }
}

#[test]
fn replay_of_every_node_s_log_after_convergence_reconstructs_it() {
    // Arrange
    let (mut cluster, lines) = logging_cluster(5, 4);
    for peer in 0..5 {
        cluster.inject_participation(peer);
    }
    for peer in 0..5 as PeerId {
        cluster.complete_local(peer);
    }
    let _ = cluster.check();

    // Act
    let departures = departures(&lines, 4);

    // Assert
    assert!(lines.iter().all(|lines| !lines.borrow().is_empty()));
    assert!(departures.is_empty(), "{departures:?}");
}
