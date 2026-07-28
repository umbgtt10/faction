// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;

pub type ChannelRegistry = Arc<Mutex<Vec<(PeerId, Sender<TransportMessage>)>>>;

pub struct ChannelsTransport {
    peer_id: PeerId,
    inbox: Receiver<TransportMessage>,
    registry: ChannelRegistry,
}

impl ChannelsTransport {
    pub fn new_mesh(peer_ids: &[PeerId]) -> Vec<ChannelsTransport> {
        let registry: ChannelRegistry = Arc::new(Mutex::new(Vec::new()));

        peer_ids
            .iter()
            .map(|&peer_id| {
                let (tx, rx) = channel();
                registry.lock().unwrap().push((peer_id, tx));
                ChannelsTransport {
                    peer_id,
                    inbox: rx,
                    registry: registry.clone(),
                }
            })
            .collect()
    }

    #[must_use]
    pub fn registry(&self) -> ChannelRegistry {
        self.registry.clone()
    }

    #[must_use]
    pub fn join_mesh(peer_id: PeerId, registry: ChannelRegistry) -> ChannelsTransport {
        let (tx, rx) = channel();
        registry.lock().unwrap().push((peer_id, tx));
        ChannelsTransport {
            peer_id,
            inbox: rx,
            registry,
        }
    }
}

impl Transport for ChannelsTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        if to == self.peer_id {
            return;
        }
        for (peer_id, sender) in self.registry.lock().unwrap().iter() {
            if *peer_id == to {
                let _ = sender.send(message.clone());
            }
        }
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        self.inbox.try_recv().ok()
    }
}
