// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Serialize;

use faction::types::PeerId;

use crate::model::fault::Fault;

// A fault in force on one node from a round until another, or until
// quiescence heals it if the chaos ends first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultWindow {
    pub peer_id: PeerId,
    pub from: u64,
    pub until: u64,
    pub fault: Fault,
}
