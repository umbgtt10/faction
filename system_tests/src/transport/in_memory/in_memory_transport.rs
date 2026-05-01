// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use faction::PeerId;
use faction_protocol::transport_message::TransportMessage;

use crate::transport::transport_trait::Transport;

type Message = (PeerId, TransportMessage);
type Inbox = Arc<Mutex<VecDeque<Message>>>;

pub struct InMemoryTransport {
    inbox: Inbox,
    outboxes: Vec<(PeerId, Inbox)>,
    local_peer_id: PeerId,
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
            .map(|(i, &local_id)| {
                let outboxes = peer_ids
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(j, &peer_id)| (peer_id, inboxes[j].clone()))
                    .collect();

                InMemoryTransport {
                    inbox: inboxes[i].clone(),
                    outboxes,
                    local_peer_id: local_id,
                }
            })
            .collect()
    }

    pub fn push_inbox(&mut self, from: PeerId, message: TransportMessage) {
        self.inbox.lock().unwrap().push_back((from, message));
    }
}

impl Transport for InMemoryTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        for (peer_id, inbox) in &self.outboxes {
            if *peer_id == to {
                inbox
                    .lock()
                    .unwrap()
                    .push_back((self.local_peer_id, message.clone()));
            }
        }
    }

    fn recv(&mut self) -> Option<(PeerId, TransportMessage)> {
        self.inbox.lock().unwrap().pop_front()
    }
}
