// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport::Transport;

pub trait LateJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport>;
}
