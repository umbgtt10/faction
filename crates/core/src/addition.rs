// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::types::PeerId;

// The addition in flight, as the view shows it: none, requested of a peer, or
// acknowledged by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Addition {
    None,
    Requested { peer_id: PeerId },
    Acknowledged { peer_id: PeerId },
}
