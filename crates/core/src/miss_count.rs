// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::types::PeerId;

// A member that has missed one or more probe periods running, and how many.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissCount {
    pub peer_id: PeerId,
    pub misses: usize,
}
