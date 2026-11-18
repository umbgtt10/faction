// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport::Transport;

pub trait LateJoinMesh {
    fn connect(&self, peer_id: u64) -> Box<dyn Transport>;
}
