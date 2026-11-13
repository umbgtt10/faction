// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::types::PeerId;

// The change in flight, as the view shows it: none, an addition requested of a
// peer or acknowledged by it, or the removal of a peer requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reconfiguration {
    None,
    AdditionRequested { peer_id: PeerId },
    AdditionAcknowledged { peer_id: PeerId },
    RemovalRequested { peer_id: PeerId },
}
