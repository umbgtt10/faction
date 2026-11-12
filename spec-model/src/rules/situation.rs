// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;

use crate::command::Command;
use crate::node_state::NodeState;
use crate::rules::change::Change;
use crate::vocabulary::command_name::CommandName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situation {
    Always,
    NotAdmitted,
    PeerIsNotMember,
    PeerIsMember,
    PeerIsPinging,
    PeerIsCollecting,
    PeerConcludes,
    NodeConcludes,
}

impl Situation {
    #[must_use]
    pub fn holds(self, command: &Command, state: &NodeState, admitted: &[CommandName]) -> bool {
        let peer_id = command.peer_id.as_ref();
        match self {
            Situation::Always => true,
            Situation::NotAdmitted => !admitted.contains(&command.name),
            Situation::PeerIsNotMember => !Self::lists(&state.members, peer_id),
            Situation::PeerIsMember => Self::lists(&state.members, peer_id),
            Situation::PeerIsPinging => Self::lists(&state.pinging_peers, peer_id),
            Situation::PeerIsCollecting => Self::lists(&state.collecting_peers, peer_id),
            Situation::PeerConcludes => {
                Self::reaches_required_count(&Change::PeerCollects.applied(command, state))
            }
            Situation::NodeConcludes => {
                Self::reaches_required_count(&Change::NodeCollects.applied(command, state))
            }
        }
    }

    fn lists(peers: &[String], peer_id: Option<&String>) -> bool {
        peer_id.is_some_and(|peer_id| peers.contains(peer_id))
    }

    fn reaches_required_count(state: &NodeState) -> bool {
        u64::try_from(state.collecting_peers.len()).is_ok_and(|count| count >= state.required_count)
    }
}
