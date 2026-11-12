// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport::Transport;

use super::late_join_mesh::LateJoinMesh;
use std::sync::Arc;

use crate::infra::network::address_book::AddressBook;
use crate::infra::transport::tcp::tcp_transport::TcpTransport;
use crate::traits::pause::Pause;

pub struct TcpJoinMesh {
    registry: AddressBook,
    pause: Arc<dyn Pause>,
}

impl TcpJoinMesh {
    #[must_use]
    pub fn new(registry: AddressBook, pause: Arc<dyn Pause>) -> Self {
        Self { registry, pause }
    }
}

impl LateJoinMesh for TcpJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(TcpTransport::join_mesh(
            peer_id,
            self.registry.clone(),
            self.pause.clone(),
        ))
    }
}
