// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::addition::Addition;
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
    pub addition: Addition,
}
