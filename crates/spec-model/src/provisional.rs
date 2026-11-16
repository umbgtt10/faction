// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde_json::Value;

use crate::command::Command;
use crate::node_state::NodeState;
use crate::vocabulary::command_name::CommandName;
use crate::vocabulary::outcome_name::OutcomeName;

// What the specification says of a provisional membership across its rows
// (§16): it ends, once and for all, when the node applies the commit of its
// own addition, adopts a membership, or is removed; and it is stated in the
// view only while it holds.
pub struct Provisional;

impl Provisional {
    // Whether the node's membership is still provisional after a command
    // answered with `outcomes`.
    #[must_use]
    pub fn after(command: &Command, outcomes: &[OutcomeName], state: &NodeState) -> bool {
        state.provisional && !Self::ends(command, outcomes, state)
    }

    // A payload with the membership stated provisional, if it is.
    #[must_use]
    pub fn stated(mut payload: Value, provisional: bool) -> Value {
        if provisional {
            payload["provisional"] = Value::Bool(true);
        }
        payload
    }

    // Whether the command's answer ends a provisional membership: an adopted
    // membership, or a commit of the node's own addition or removal applied.
    fn ends(command: &Command, outcomes: &[OutcomeName], state: &NodeState) -> bool {
        let commits_the_node = matches!(
            command.name,
            CommandName::AdditionCommitted | CommandName::RemovalCommitted
        ) && command.peer_id.as_ref() == Some(&state.peer_id);
        outcomes.contains(&OutcomeName::MembershipAdopted)
            || commits_the_node && !outcomes.contains(&OutcomeName::CommitIgnored)
    }
}
