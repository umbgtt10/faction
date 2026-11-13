// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;
use tonic::{Request, Response, Status};

use crate::infra::transport::grpc::grpc_proto::wire::Envelope;
use crate::infra::transport::grpc::grpc_proto::wire::transport_server::Transport as TransportService;

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
        let d = r.into_inner().data;
        if d.len() == 9 {
            let from = PeerId::from_le_bytes(d[0..8].try_into().unwrap());
            let msg = match d[8] {
                0 => TransportMessage::Ping { from },
                1 => TransportMessage::Ready { from },
                2 => TransportMessage::Bootstrapped { from },
                _ => return Ok(Response::new(Envelope { data: vec![] })),
            };
            self.inbox.lock().unwrap().push_back(msg);
        }
        Ok(Response::new(Envelope { data: vec![] }))
    }
}
