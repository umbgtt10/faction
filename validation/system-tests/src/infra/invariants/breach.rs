// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::types::PeerId;

// An invariant a node broke: the node, the invariant, and the command it
// broke it on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breach {
    pub peer_id: PeerId,
    pub invariant: &'static str,
    pub command: Command,
}
