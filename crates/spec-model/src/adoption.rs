// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::command::Command;
use crate::node_state::NodeState;
use crate::reconfiguration::Reconfiguration;

// What the specification says a node does with a membership it adopts (§16):
// its members, required count and epoch are the membership's; every list it
// holds keeps only the new members; the member probed last, no longer one, is
// forgotten; and no change is in flight.
pub struct Adoption;

impl Adoption {
    #[must_use]
    pub fn adopted(command: &Command, state: &NodeState) -> NodeState {
        let members = command.members.clone();
        NodeState {
            pinging_peers: Self::kept(&state.pinging_peers, &members),
            collecting_peers: Self::kept(&state.collecting_peers, &members),
            suspected: Self::kept(&state.suspected, &members),
            misses: state
                .misses
                .iter()
                .filter(|(peer, _)| members.contains(peer))
                .map(|(peer, misses)| (peer.clone(), *misses))
                .collect(),
            last_probed: state
                .last_probed
                .clone()
                .filter(|last| members.contains(last)),
            reconfiguration: Reconfiguration::None,
            required_count: command.required_count.expect("a required count"),
            epoch: command.epoch,
            members,
            ..state.clone()
        }
    }

    fn kept(peers: &[String], members: &[String]) -> Vec<String> {
        peers
            .iter()
            .filter(|peer| members.contains(peer))
            .cloned()
            .collect()
    }
}
