// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;
use core::iter::once;

use faction::command::Command;
use faction::config::Config;
use faction::quorum_policy::QuorumPolicy;
use faction::types::Epoch;
use faction::types::PeerId;

use crate::output_message::OutputMessage;
use crate::transport_message::TransportMessage;

// A membership a peer sends a node, as the node takes it: as a newcomer does,
// its members and the node itself, the required count and their epoch -- a
// member's reply to a claim from behind, or a coordinator's invitation.
pub struct Offer {
    from: PeerId,
    members: Vec<PeerId>,
    required_count: usize,
    epoch: Epoch,
}

impl Offer {
    // The membership `message` carries, as the node `peer_id` takes it; none
    // for any other message.
    #[must_use]
    pub fn of(message: &TransportMessage, peer_id: PeerId) -> Option<Self> {
        match message {
            TransportMessage::Membership {
                from,
                members,
                required_count,
                epoch,
            } => Some(Self {
                from: *from,
                members: members
                    .iter()
                    .copied()
                    .chain((!members.contains(&peer_id)).then_some(peer_id))
                    .collect(),
                required_count: *required_count,
                epoch: *epoch,
            }),
            _ => None,
        }
    }

    // What the node does once it has taken the offer: the offer acknowledged,
    // at `epoch`, the node's after, then `taken`, what taking it asked.
    #[must_use]
    pub fn acknowledged(
        &self,
        taken: Vec<OutputMessage>,
        peer_id: PeerId,
        epoch: Epoch,
    ) -> Vec<OutputMessage> {
        once(OutputMessage::Send {
            to: self.from,
            message: TransportMessage::MembershipAcknowledged {
                from: peer_id,
                epoch,
            },
        })
        .chain(
            taken
                .into_iter()
                .filter(|output| *output != OutputMessage::Noop),
        )
        .collect()
    }

    // The command a running node is given for the offer.
    #[must_use]
    pub fn command(&self) -> Command {
        Command::MembershipReceived {
            members: self.members.clone(),
            required_count: self.required_count,
            epoch: self.epoch,
        }
    }

    // The configuration a newcomer `peer_id` is created from.
    #[must_use]
    pub fn config(&self, peer_id: PeerId, suspicion_threshold: usize) -> Config {
        Config::new(
            peer_id,
            self.members.clone(),
            QuorumPolicy::new(self.required_count),
            suspicion_threshold,
            self.epoch,
        )
    }
}
