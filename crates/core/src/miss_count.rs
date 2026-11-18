// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance::peer_id::PeerId;

// A member that has missed one or more probe periods running, and how many.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissCount<P: PeerId = u64> {
    pub peer_id: P,
    pub misses: usize,
}
