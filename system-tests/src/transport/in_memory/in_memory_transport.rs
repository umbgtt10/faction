// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;

use faction_protocol::transport_trait::Transport;

pub type Inbox = Arc<Mutex<VecDeque<TransportMessage>>>;
pub type Registry = Arc<Mutex<Vec<(PeerId, Inbox)>>>;

pub struct InMemoryTransport {
    peer_id: PeerId,
    inbox: Inbox,
    registry: Registry,
}

impl InMemoryTransport {
    pub fn new_mesh(peer_ids: &[PeerId]) -> Vec<InMemoryTransport> {
        let registry: Registry = Arc::new(Mutex::new(Vec::new()));

        peer_ids
            .iter()
            .map(|&peer_id| {
                let inbox: Inbox = Arc::new(Mutex::new(VecDeque::new()));
                registry.lock().unwrap().push((peer_id, inbox.clone()));
                InMemoryTransport {
                    peer_id,
                    inbox,
                    registry: registry.clone(),
                }
            })
            .collect()
    }

    #[must_use]
    pub fn registry(&self) -> Registry {
        self.registry.clone()
    }

    #[must_use]
    pub fn join_mesh(peer_id: PeerId, registry: Registry) -> InMemoryTransport {
        let inbox: Inbox = Arc::new(Mutex::new(VecDeque::new()));
        registry.lock().unwrap().push((peer_id, inbox.clone()));
        InMemoryTransport {
            peer_id,
            inbox,
            registry,
        }
    }

    pub fn push_inbox(&mut self, message: TransportMessage) {
        self.inbox.lock().unwrap().push_back(message);
    }
}

impl Transport for InMemoryTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        if to == self.peer_id {
            return;
        }
        for (peer_id, inbox) in self.registry.lock().unwrap().iter() {
            if *peer_id == to {
                inbox.lock().unwrap().push_back(message.clone());
            }
        }
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        self.inbox.lock().unwrap().pop_front()
    }
}
