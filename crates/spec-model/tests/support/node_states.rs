// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_spec_model::addition::Addition;
use faction_spec_model::node_state::NodeState;
use faction_spec_model::vocabulary::stage::Stage;

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
            addition: Addition::None,
        }
    }

    // Concluded with an addition in flight, the node itself its one collecting
    // peer.
    pub fn reconfiguring(addition: Addition) -> NodeState {
        NodeState {
            addition,
            ..Self::of(Stage::Reconfiguring, &[], &["1"])
        }
    }

    pub fn peers(peers: &[&str]) -> Vec<String> {
        peers.iter().map(|peer| String::from(*peer)).collect()
    }
}
