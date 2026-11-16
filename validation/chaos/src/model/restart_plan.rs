// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Serialize;

use faction::types::PeerId;

use crate::model::restart_from::RestartFrom;

// A node crashed at a round and down for a while -- it sends nothing,
// receives nothing, and its timers die -- then restarted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestartPlan {
    pub peer_id: PeerId,
    pub at: u64,
    pub down_for: u64,
    pub from: RestartFrom,
}
