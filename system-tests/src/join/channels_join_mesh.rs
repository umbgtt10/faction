// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport_trait::Transport;

use super::late_join_mesh::LateJoinMesh;
use crate::transport::channels::channels_transport::ChannelRegistry;
use crate::transport::channels::channels_transport::ChannelsTransport;

pub struct ChannelsJoinMesh {
    registry: ChannelRegistry,
}

impl ChannelsJoinMesh {
    #[must_use]
    pub fn new(registry: ChannelRegistry) -> Self {
        Self { registry }
    }
}

impl LateJoinMesh for ChannelsJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(ChannelsTransport::join_mesh(peer_id, self.registry.clone()))
    }
}
