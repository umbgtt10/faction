// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;

use faction_protocol::transport_trait::Transport;

type Message = TransportMessage;
type Inbox = Arc<Mutex<VecDeque<Message>>>;

pub struct InMemoryTransport {
    inbox: Inbox,
    outboxes: Vec<(PeerId, Inbox)>,
}

impl Drop for InMemoryTransport {
    fn drop(&mut self) {}
}

impl InMemoryTransport {
    pub fn new_mesh(peer_ids: &[PeerId]) -> Vec<InMemoryTransport> {
        let inboxes: Vec<_> = peer_ids
            .iter()
            .map(|_| Arc::new(Mutex::new(VecDeque::new())))
            .collect();

        peer_ids
            .iter()
            .enumerate()
            .map(|(i, &_)| {
                let outboxes = peer_ids
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(j, &peer_id)| (peer_id, inboxes[j].clone()))
                    .collect();

                InMemoryTransport {
                    inbox: inboxes[i].clone(),
                    outboxes,
                }
            })
            .collect()
    }

    pub fn push_inbox(&mut self, message: TransportMessage) {
        self.inbox.lock().unwrap().push_back(message);
    }
}

impl Transport for InMemoryTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        for (peer_id, inbox) in &self.outboxes {
            if *peer_id == to {
                inbox.lock().unwrap().push_back(message.clone());
            }
        }
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        self.inbox.lock().unwrap().pop_front()
    }
}
