// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

// A change of the cluster the layer above carries through: a peer added --
// a newcomer, or a removed node taken back -- a member removed, or a new
// required count, which the layer above commits without a coordinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Addition { peer_id: PeerId },
    Removal { peer_id: PeerId },
    RequiredCount { required_count: usize },
}
