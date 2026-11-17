// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::node_state::NodeState;
use crate::vocabulary::stage::Stage;

const CHANGES_IN_FLIGHT: &str = "changes-in-flight";
const UNCHANGED: &str = "unchanged";

// What a row says of the stage after: kept, a stage named, or the one the
// changes in flight then make (§2) -- none Bootstrapped, fewer than the bound
// Coordinating, as many Reconfiguring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    Unchanged,
    Stage(Stage),
    ChangesInFlight,
}

impl Next {
    #[must_use]
    pub fn of(name: &str) -> Self {
        match name {
            UNCHANGED => Next::Unchanged,
            CHANGES_IN_FLIGHT => Next::ChangesInFlight,
            stage => Next::Stage(Stage::of(stage)),
        }
    }

    // The stage after a row's change, `changed` the node it leaves.
    #[must_use]
    pub fn stage(self, changed: &NodeState) -> Stage {
        match self {
            Next::Unchanged => changed.stage,
            Next::Stage(stage) => stage,
            Next::ChangesInFlight => {
                let in_flight = changed.reconfigurations.len() as u64;
                if in_flight == 0 {
                    Stage::Bootstrapped
                } else if in_flight < changed.changes_in_flight {
                    Stage::Coordinating
                } else {
                    Stage::Reconfiguring
                }
            }
        }
    }
}
