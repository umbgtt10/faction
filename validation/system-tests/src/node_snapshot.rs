// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::reconfiguration::Reconfiguration;
use faction::types::Epoch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSnapshot {
    pub peer_state: PeerState,
    pub member_count: usize,
    pub members: Vec<u64>,
    pub required_count: usize,
    pub reconfigurations: Vec<Reconfiguration>,
    pub suspected: Vec<u64>,
    pub quorum: Quorum,
    pub epoch: Epoch,
}
