// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport_trait::Transport;

use super::late_join_mesh::LateJoinMesh;
use crate::transport::in_memory::in_memory_transport::InMemoryTransport;
use crate::transport::in_memory::in_memory_transport::Registry;

pub struct InMemoryJoinMesh {
    registry: Registry,
}

impl InMemoryJoinMesh {
    #[must_use]
    pub fn new(registry: Registry) -> Self {
        Self { registry }
    }
}

impl LateJoinMesh for InMemoryJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(InMemoryTransport::join_mesh(peer_id, self.registry.clone()))
    }
}
