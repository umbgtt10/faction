// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::iter::once;

use faction::config::Config;
use faction::faction::Faction;
use faction::observer::Observer;
use faction::quorum_policy::QuorumPolicy;
use faction::types::PeerId;

use crate::output_message::OutputMessage;
use crate::protocol::Protocol;
use crate::transport_message::TransportMessage;

// A peer being added, before it has a configuration: it waits for the
// membership its coordinator sends, and becomes a node of it -- its own id,
// the members, itself among them, and the required count -- acknowledging the
// membership as it starts.
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
        match message {
            TransportMessage::Membership {
                from,
                members,
                required_count,
            } => {
                let joins = !members.contains(&self.peer_id);
                let peers: Vec<PeerId> = members
                    .into_iter()
                    .chain(joins.then_some(self.peer_id))
                    .collect();
                let config = Config::new(
                    self.peer_id,
                    peers.clone(),
                    QuorumPolicy::new(required_count),
                    self.suspicion_threshold,
                );
                let protocol = Protocol::new(
                    Faction::new(config, self.observer),
                    peers,
                    self.peer_id,
                    self.helpers,
                );
                let acknowledgement = OutputMessage::Send {
                    to: from,
                    message: TransportMessage::MembershipAcknowledged { from: self.peer_id },
                };
                let outputs = once(acknowledgement).chain(protocol.initialize()).collect();
                Ok((protocol, outputs))
            }
            _ => Err(self),
        }
    }
}
