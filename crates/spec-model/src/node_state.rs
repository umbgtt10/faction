// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::reconfiguration::Reconfiguration;
use crate::vocabulary::stage::Stage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeState {
    pub peer_id: String,
    pub required_count: u64,
    pub stage: Stage,
    pub members: Vec<String>,
    pub pinging_peers: Vec<String>,
    pub collecting_peers: Vec<String>,
    pub deadline_missed: bool,
    pub reconfiguration: Reconfiguration,
    pub completed_before_removal: bool,
    pub suspicion_threshold: u64,
    pub suspected: Vec<String>,
    pub misses: BTreeMap<String, u64>,
    pub last_probed: Option<String>,
}
