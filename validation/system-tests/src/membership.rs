// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::Epoch;
use faction::types::PeerId;

// A membership a node is created at: its peers, the count they require and
// their epoch -- a genesis at epoch 0, or a snapshot a restart keeps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    pub peers: Vec<PeerId>,
    pub required_count: usize,
    pub epoch: Epoch,
}
