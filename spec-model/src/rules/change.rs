// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::command::Command;
use crate::node_state::NodeState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Nothing,
    PeerPings,
    PeerCollects,
    NodeCollects,
    MissesDeadline,
    PeerJoins,
}

impl Change {
    #[must_use]
    pub fn applied(self, command: &Command, state: &NodeState) -> NodeState {
        let peer_id = command.peer_id.clone();
        match self {
            Change::Nothing => state.clone(),
            Change::PeerPings => NodeState {
                pinging_peers: Self::appended(&state.pinging_peers, peer_id),
                ..state.clone()
            },
            Change::PeerCollects => NodeState {
                collecting_peers: Self::appended(&state.collecting_peers, peer_id),
                ..state.clone()
            },
            Change::NodeCollects => NodeState {
                collecting_peers: Self::appended(
                    &state.collecting_peers,
                    (!state.collecting_peers.contains(&state.peer_id))
                        .then(|| state.peer_id.clone()),
                ),
                ..state.clone()
            },
            Change::MissesDeadline => NodeState {
                deadline_missed: true,
                ..state.clone()
            },
            Change::PeerJoins => NodeState {
                members: Self::appended(&state.members, peer_id),
                ..state.clone()
            },
        }
    }

    fn appended(peers: &[String], peer_id: Option<String>) -> Vec<String> {
        peers.iter().cloned().chain(peer_id).collect()
    }
}
