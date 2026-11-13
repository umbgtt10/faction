// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeSnapshot {
    pub peer_state: PeerState,
    pub member_count: usize,
}
