// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use faction::PeerId;
use faction_protocol::transport_message::TransportMessage;

use crate::transport::transport_trait::Transport;

pub struct InMemoryTransport {
    inbox: Arc<Mutex<VecDeque<(PeerId, TransportMessage)>>>,
    outboxes: Vec<(PeerId, Arc<Mutex<VecDeque<(PeerId, TransportMessage)>>>)>,
    local_peer_id: PeerId,
}

impl InMemoryTransport {
    pub fn new_pair(peer_a: PeerId, peer_b: PeerId) -> (InMemoryTransport, InMemoryTransport) {
        let inbox_a = Arc::new(Mutex::new(VecDeque::new()));
        let inbox_b = Arc::new(Mutex::new(VecDeque::new()));

        let transport_a = InMemoryTransport {
            inbox: inbox_a.clone(),
            outboxes: vec![(peer_b, inbox_b.clone())],
            local_peer_id: peer_a,
        };

        let transport_b = InMemoryTransport {
            inbox: inbox_b,
            outboxes: vec![(peer_a, inbox_a)],
            local_peer_id: peer_b,
        };

        (transport_a, transport_b)
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
