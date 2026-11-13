// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::addition::Addition;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSnapshot {
    pub peer_state: PeerState,
    pub member_count: usize,
    pub required_count: usize,
    pub addition: Addition,
    pub suspected: Vec<PeerId>,
    pub quorum: Quorum,
}
