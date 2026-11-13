// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use faction_protocol::transport_message::TransportMessage;
use tonic::{Request, Response, Status};

use crate::infra::transport::grpc::grpc_proto::wire::Envelope;
use crate::infra::transport::grpc::grpc_proto::wire::transport_server::Transport as TransportService;
use crate::infra::transport::wire_codec::WireCodec;

type Inbox = Arc<Mutex<VecDeque<TransportMessage>>>;

pub struct GrpcService {
    inbox: Inbox,
}

impl GrpcService {
    #[must_use]
    pub fn new(inbox: Inbox) -> Self {
        Self { inbox }
    }
}

#[tonic::async_trait]
impl TransportService for GrpcService {
    async fn deliver(&self, r: Request<Envelope>) -> Result<Response<Envelope>, Status> {
        if let Some(message) = WireCodec::decode(&r.into_inner().data) {
            self.inbox.lock().unwrap().push_back(message);
        }
        Ok(Response::new(Envelope { data: vec![] }))
    }
}
