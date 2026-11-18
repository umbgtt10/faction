// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Serialize;

// A change of the cluster the layer above carries through: a peer added --
// a newcomer, or a removed node taken back -- a member removed, or a new
// required count, which the layer above commits without a coordinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    Addition { peer_id: u64 },
    Removal { peer_id: u64 },
    RequiredCount { required_count: usize },
}
