// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance::peer_id::PeerId;

// A change in flight, as the view shows it: an addition requested of a peer or
// acknowledged by it, or the removal of a peer requested, with the count its
// request stated it leaves, if it stated one (§12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reconfiguration<P: PeerId = u64> {
    AdditionRequested { peer_id: P },
    AdditionAcknowledged { peer_id: P },
    RemovalRequested { peer_id: P, leaves: Option<usize> },
}
