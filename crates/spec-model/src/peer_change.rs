// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::node_state::NodeState;
use crate::reconfiguration::Reconfiguration;
use crate::vocabulary::command_name::CommandName;

// What the specification says of the change in flight naming a command's peer
// (§11, §12, §17): whether there is one, of either kind; whether it is an
// addition requested, its acknowledgement awaited; whether it is a proposal, an
// addition acknowledged or a removal requested, its decline awaited; and
// whether it is the change the command commits or aborts.
pub struct PeerChange;

impl PeerChange {
    #[must_use]
    pub fn awaits_acknowledgement(command: &Command, state: &NodeState) -> bool {
        let peer_id = command.peer_id.as_ref();
        state.reconfigurations.iter().any(
            |change| matches!(change, Reconfiguration::AdditionRequested(pending) if Some(pending) == peer_id),
        )
    }

    #[must_use]
    pub fn awaits_decline(command: &Command, state: &NodeState) -> bool {
        let peer_id = command.peer_id.as_ref();
        state.reconfigurations.iter().any(|change| {
            matches!(
                change,
                Reconfiguration::AdditionAcknowledged(proposed)
                    | Reconfiguration::RemovalRequested(proposed, _) if Some(proposed) == peer_id
            )
        })
    }

    #[must_use]
    pub fn in_flight(command: &Command, state: &NodeState) -> bool {
        let peer_id = command.peer_id.as_ref();
        state.reconfigurations.iter().any(|change| match change {
            Reconfiguration::AdditionRequested(named)
            | Reconfiguration::AdditionAcknowledged(named)
            | Reconfiguration::RemovalRequested(named, _) => Some(named) == peer_id,
        })
    }

    // A removal's of the command's peer for a removal's command, an
    // addition's otherwise.
    #[must_use]
    pub fn pending(command: &Command, state: &NodeState) -> bool {
        let peer_id = command.peer_id.as_ref();
        let removal = matches!(
            command.name,
            CommandName::RemovalCommitted | CommandName::RemovalAborted
        );
        state.reconfigurations.iter().any(|change| match change {
            Reconfiguration::RemovalRequested(pending, _) => removal && Some(pending) == peer_id,
            Reconfiguration::AdditionRequested(pending)
            | Reconfiguration::AdditionAcknowledged(pending) => {
                !removal && Some(pending) == peer_id
            }
        })
    }
}
