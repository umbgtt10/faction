// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::faction::Faction;
use faction::observer::Observer;
use faction::types::PeerId;

use crate::offer::Offer;
use crate::output_message::OutputMessage;
use crate::protocol::Protocol;
use crate::transport_message::TransportMessage;

// A peer being added, before it has a configuration: it waits for the
// membership its coordinator sends, and becomes a node of it -- its own id,
// the members, itself among them, the required count and their epoch --
// acknowledging the membership as it starts. A removed node invited again
// becomes one.
pub struct Newcomer {
    peer_id: PeerId,
    suspicion_threshold: usize,
    helpers: usize,
    observer: Box<dyn Observer>,
}

impl Newcomer {
    // Its own id, and the tolerance its deployment gives it: the suspicion
    // threshold its node is created with, and how many helpers it asks.
    #[must_use]
    pub fn new(
        peer_id: PeerId,
        suspicion_threshold: usize,
        helpers: usize,
        observer: Box<dyn Observer>,
    ) -> Self {
        Self {
            peer_id,
            suspicion_threshold,
            helpers,
            observer,
        }
    }

    // The node the membership makes of the newcomer, and what it does first;
    // the newcomer itself, still waiting, given anything else.
    pub fn join(self, message: TransportMessage) -> Result<(Protocol, Vec<OutputMessage>), Self> {
        let Some(offer) = Offer::of(&message, self.peer_id) else {
            return Err(self);
        };
        let config = offer.config(self.peer_id, self.suspicion_threshold);
        let epoch = config.epoch();
        let peers = config.peers().to_vec();
        let mut protocol = Protocol::new(
            Faction::new(config, self.observer),
            peers,
            self.peer_id,
            self.helpers,
        );
        let opening = protocol.initialize();
        let outputs = offer.acknowledged(opening, self.peer_id, epoch);
        Ok((protocol, outputs))
    }
}
