// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::addition::Addition;
use crate::command::Command;
use crate::liveness::Liveness;
use crate::node_state::NodeState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Nothing,
    PeerPings,
    PeerCollects,
    NodeCollects,
    MissesDeadline,
    PeerJoins,
    AdditionPends,
    AdditionAcknowledged,
    AdditionWithdrawn,
    AdditionCompletes,
    CountSet,
    ProbeMoves,
    PeerRevived,
    MissesCleared,
    MissCounted,
    PeerSuspected,
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
            Change::AdditionPends => NodeState {
                addition: Addition::Requested(Self::named(peer_id)),
                ..state.clone()
            },
            Change::AdditionAcknowledged => NodeState {
                addition: Addition::Acknowledged(Self::named(peer_id)),
                ..state.clone()
            },
            Change::AdditionWithdrawn => NodeState {
                addition: Addition::None,
                ..state.clone()
            },
            Change::AdditionCompletes => NodeState {
                members: Self::appended(&state.members, peer_id),
                addition: Addition::None,
                ..state.clone()
            },
            Change::CountSet => NodeState {
                required_count: command.required_count.expect("a required count"),
                ..state.clone()
            },
            Change::ProbeMoves => NodeState {
                last_probed: Liveness::next_to_probe(state),
                ..state.clone()
            },
            Change::PeerRevived => NodeState {
                suspected: Self::without(&state.suspected, &Self::named(peer_id)),
                ..state.clone()
            },
            Change::MissesCleared => NodeState {
                misses: Self::forgotten(state, &Self::named(peer_id)),
                ..state.clone()
            },
            Change::MissCounted => {
                let peer_id = Self::named(peer_id);
                let misses = Liveness::misses(state, &peer_id) + 1;
                let mut counted = state.misses.clone();
                counted.insert(peer_id, misses);
                NodeState {
                    misses: counted,
                    ..state.clone()
                }
            }
            Change::PeerSuspected => {
                let peer_id = Self::named(peer_id);
                NodeState {
                    misses: Self::forgotten(state, &peer_id),
                    suspected: Self::appended(&state.suspected, Some(peer_id)),
                    ..state.clone()
                }
            }
        }
    }

    fn appended(peers: &[String], peer_id: Option<String>) -> Vec<String> {
        peers.iter().cloned().chain(peer_id).collect()
    }

    fn forgotten(state: &NodeState, peer_id: &str) -> BTreeMap<String, u64> {
        let mut misses = state.misses.clone();
        misses.remove(peer_id);
        misses
    }

    fn named(peer_id: Option<String>) -> String {
        peer_id.expect("a peer")
    }

    fn without(peers: &[String], peer_id: &str) -> Vec<String> {
        peers
            .iter()
            .filter(|peer| *peer != peer_id)
            .cloned()
            .collect()
    }
}
