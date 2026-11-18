// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::faction::Faction;
use faction::instance::instance_id::InstanceId;
use faction::observer::Observer;

use crate::offer::Offer;
use crate::output_message::OutputMessage;
use crate::protocol::Protocol;
use crate::transport_message::TransportMessage;

// A peer being added, before it has a configuration: it waits for the
// membership its coordinator sends, and becomes a node of it -- its instance,
// the members, itself among them, the required count and their epoch --
// acknowledging the membership as it starts. A removed node invited again
// becomes one.
pub struct Newcomer {
    instance: InstanceId,
    suspicion_threshold: usize,
    changes_in_flight: usize,
    helpers: usize,
    observer: Box<dyn Observer>,
}

impl Newcomer {
    // The instance it becomes -- its node's id and an incarnation the caller
    // gives it, new to its node -- and the tolerance its deployment gives it:
    // the suspicion threshold its node is created with, its bound of changes
    // in flight, and how many helpers it asks.
    #[must_use]
    pub fn new(
        instance: InstanceId,
        suspicion_threshold: usize,
        changes_in_flight: usize,
        helpers: usize,
        observer: Box<dyn Observer>,
    ) -> Self {
        Self {
            instance,
            suspicion_threshold,
            changes_in_flight,
            helpers,
            observer,
        }
    }

    // The node the membership makes of the newcomer, and what it does first;
    // the newcomer itself, still waiting, given anything else.
    pub fn join(self, message: TransportMessage) -> Result<(Protocol, Vec<OutputMessage>), Self> {
        let peer_id = self.instance.node;
        let Some(offer) = Offer::of(&message, peer_id) else {
            return Err(self);
        };
        let config = offer.config(
            self.instance,
            self.suspicion_threshold,
            self.changes_in_flight,
        );
        let epoch = config.epoch();
        let peers = config.peers().to_vec();
        let mut protocol = Protocol::new(
            Faction::new(config, self.observer),
            peers,
            peer_id,
            self.helpers,
        );
        let opening = protocol.initialize();
        let outputs = offer.acknowledged(opening, peer_id, epoch);
        Ok((protocol, outputs))
    }
}
