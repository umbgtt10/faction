// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;
use faction::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSnapshot {
    pub peer_state: PeerState,
    pub member_count: usize,
    pub required_count: usize,
    pub addition: Reconfiguration,
    pub suspected: Vec<PeerId>,
    pub quorum: Quorum,
}
