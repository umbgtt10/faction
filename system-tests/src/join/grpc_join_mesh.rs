// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport::Transport;

use super::late_join_mesh::LateJoinMesh;
use crate::infra::network::address_book::AddressBook;
use crate::infra::transport::grpc::grpc_transport::GrpcTransport;

pub struct GrpcJoinMesh {
    registry: AddressBook,
}

impl GrpcJoinMesh {
    #[must_use]
    pub fn new(registry: AddressBook) -> Self {
        Self { registry }
    }
}

impl LateJoinMesh for GrpcJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(GrpcTransport::join_mesh(peer_id, self.registry.clone()))
    }
}
