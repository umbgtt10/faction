// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport_trait::Transport;

use super::late_join_mesh::LateJoinMesh;
use crate::transport::tcp::tcp_transport::AddressBook;
use crate::transport::tcp::tcp_transport::TcpTransport;

pub struct TcpJoinMesh {
    registry: AddressBook,
}

impl TcpJoinMesh {
    #[must_use]
    pub fn new(registry: AddressBook) -> Self {
        Self { registry }
    }
}

impl LateJoinMesh for TcpJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(TcpTransport::join_mesh(peer_id, self.registry.clone()))
    }
}
