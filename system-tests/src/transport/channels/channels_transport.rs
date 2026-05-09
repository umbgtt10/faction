// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::mpsc::{Receiver, Sender, channel};

use faction::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;

pub struct ChannelsTransport {
    inbox: Receiver<TransportMessage>,
    outboxes: Vec<(PeerId, Sender<TransportMessage>)>,
}

impl Drop for ChannelsTransport {
    fn drop(&mut self) {}
}

impl ChannelsTransport {
    pub fn new_mesh(peer_ids: &[PeerId]) -> Vec<ChannelsTransport> {
        let n = peer_ids.len();
        let mut inbox_senders: Vec<Sender<TransportMessage>> = Vec::new();
        let mut inbox_receivers: Vec<Receiver<TransportMessage>> = Vec::new();

        for _ in 0..n {
            let (tx, rx) = channel();
            inbox_senders.push(tx);
            inbox_receivers.push(rx);
        }

        peer_ids
            .iter()
            .enumerate()
            .map(|(i, &_local_id)| {
                let outboxes: Vec<_> = peer_ids
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(j, &peer_id)| (peer_id, inbox_senders[j].clone()))
                    .collect();

                ChannelsTransport {
                    inbox: inbox_receivers.remove(0),
                    outboxes,
                }
            })
            .collect()
    }
}

impl Transport for ChannelsTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        for (peer_id, sender) in &self.outboxes {
            if *peer_id == to {
                let _ = sender.send(message.clone());
            }
        }
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        self.inbox.try_recv().ok()
    }
}
