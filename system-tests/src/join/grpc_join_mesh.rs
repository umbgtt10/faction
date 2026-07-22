// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport_trait::Transport;

use super::late_join_mesh::LateJoinMesh;
use crate::transport::grpc::grpc_transport::AddressBook as GrpcAddressBook;
use crate::transport::grpc::grpc_transport::GrpcTransport;

pub struct GrpcJoinMesh {
    registry: GrpcAddressBook,
}

impl GrpcJoinMesh {
    #[must_use]
    pub fn new(registry: GrpcAddressBook) -> Self {
        Self { registry }
    }
}

impl LateJoinMesh for GrpcJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(GrpcTransport::join_mesh(peer_id, self.registry.clone()))
    }
}
