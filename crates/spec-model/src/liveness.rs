// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::node_state::NodeState;

// What the specification says of a concluded node's members' liveness (§13,
// §14): whom it probes next, whom it lists to probe a member that missed, how
// many periods running a member has missed, and whether it holds its quorum.
pub struct Liveness;

impl Liveness {
    // Every member not suspected but the node and the missed one, from the
    // member after the missed one, past the last back to the first.
    #[must_use]
    pub fn helpers(state: &NodeState, missed: &str) -> Vec<String> {
        Self::after(&state.members, Some(missed))
            .filter(|member| {
                **member != state.peer_id
                    && member.as_str() != missed
                    && !state.suspected.contains(member)
            })
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn holds_quorum(state: &NodeState) -> bool {
        let alive = state
            .members
            .iter()
            .filter(|member| !state.suspected.contains(member))
            .count();
        u64::try_from(alive).is_ok_and(|alive| alive >= state.required_count)
    }

    #[must_use]
    pub fn misses(state: &NodeState, peer_id: &str) -> u64 {
        state.misses.get(peer_id).copied().unwrap_or(0)
    }

    // The member after the one probed last, past the last back to the first,
    // the node skipped; the first member that is not the node, if it has
    // probed no one.
    #[must_use]
    pub fn next_to_probe(state: &NodeState) -> Option<String> {
        Self::after(&state.members, state.last_probed.as_deref())
            .find(|member| **member != state.peer_id)
            .cloned()
    }

    // Every member once, starting after `from` -- from the first, with no
    // `from` or one that is no member.
    fn after<'a>(members: &'a [String], from: Option<&str>) -> impl Iterator<Item = &'a String> {
        let start = from
            .and_then(|from| members.iter().position(|member| member == from))
            .map_or(0, |position| position + 1);
        members.iter().cycle().skip(start).take(members.len())
    }
}
