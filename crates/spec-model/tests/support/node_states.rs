// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_spec_model::node_state::NodeState;
use faction_spec_model::reconfiguration::Reconfiguration;
use faction_spec_model::vocabulary::stage::Stage;
use std::collections::BTreeMap;

// Node 1 of a cluster of members 1, 2 and 3 that requires two, in the given
// stage, with the given pinging and collecting peers.
pub struct NodeStates;

impl NodeStates {
    pub fn of(stage: Stage, pinging_peers: &[&str], collecting_peers: &[&str]) -> NodeState {
        NodeState {
            peer_id: String::from("1"),
            required_count: 2,
            stage,
            members: Self::peers(&["1", "2", "3"]),
            pinging_peers: Self::peers(pinging_peers),
            collecting_peers: Self::peers(collecting_peers),
            deadline_missed: false,
            reconfiguration: Reconfiguration::None,
            completed_before_removal: false,
            suspicion_threshold: 1,
            suspected: Vec::new(),
            misses: BTreeMap::new(),
            last_probed: None,
        }
    }

    // Concluded, the node itself its one collecting peer, every member alive.
    pub fn concluded() -> NodeState {
        Self::of(Stage::Bootstrapped, &[], &["1"])
    }

    pub fn misses(misses: &[(&str, u64)]) -> BTreeMap<String, u64> {
        misses
            .iter()
            .map(|(peer, count)| (String::from(*peer), *count))
            .collect()
    }

    // Concluded with a change in flight, the node itself its one collecting
    // peer.
    pub fn reconfiguring(reconfiguration: Reconfiguration) -> NodeState {
        NodeState {
            reconfiguration,
            ..Self::of(Stage::Reconfiguring, &[], &["1"])
        }
    }

    pub fn peers(peers: &[&str]) -> Vec<String> {
        peers.iter().map(|peer| String::from(*peer)).collect()
    }
}
