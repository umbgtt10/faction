// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::Epoch;

// A change a node proposed to the layer above, at the epoch it decided it at
// (§17).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proposal {
    Addition { peer_id: u64, epoch: Epoch },
    Removal { peer_id: u64, epoch: Epoch },
}
