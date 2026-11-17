// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;
use core::iter::once;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::instance::instance_id::InstanceId;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;
use faction::types::Epoch;
use faction::types::PeerId;

use crate::output_message::OutputMessage;
use crate::transport_message::TransportMessage;

// A membership a peer sends a node -- a member's reply to a claim from behind,
// or a coordinator's invitation -- its members as sent, the required count and
// their epoch. A running node takes it as sent: one that does not name the
// node removes it (§16). A newcomer takes it with itself among the members.
// Only a membership addressed to the node is one: another node's, delivered to
// it, is no offer, so a misdelivered invitation brings no one back.
pub struct Offer {
    from: PeerId,
    members: Vec<PeerId>,
    required_count: usize,
    epoch: Epoch,
}

impl Offer {
    // The membership `message` carries, as the node `peer_id` takes it; none
    // for a membership addressed to another peer, nor for any other message.
    #[must_use]
    pub fn of(message: &TransportMessage, peer_id: PeerId) -> Option<Self> {
        match message {
            TransportMessage::Membership {
                from,
                to,
                members,
                required_count,
                epoch,
            } if *to == peer_id => Some(Self {
                from: *from,
                members: members.clone(),
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

    // What a running node does once it has taken the offer, as `view` shows
    // it after: the offer acknowledged at its epoch, then `taken` -- but by a
    // node it removed, which has nothing to acknowledge: the invitation that
    // brings it back reaches it removed.
    #[must_use]
    pub fn answered(
        &self,
        taken: Vec<OutputMessage>,
        peer_id: PeerId,
        view: &ClusterView,
    ) -> Vec<OutputMessage> {
        if view.peer_state() == PeerState::Removed {
            taken
        } else {
            self.acknowledged(taken, peer_id, view.epoch())
        }
    }

    // The command a running node is given for the offer: the membership as
    // sent.
    #[must_use]
    pub fn command(&self) -> Command {
        Command::MembershipReceived {
            members: self.members.clone(),
            required_count: self.required_count,
            epoch: self.epoch,
        }
    }

    // The configuration a newcomer is created from: the offer's members and
    // the newcomer itself, its membership provisional, its own addition yet to
    // commit.
    #[must_use]
    pub fn config(&self, instance: InstanceId, suspicion_threshold: usize) -> Config {
        Config::new(
            instance,
            self.members
                .iter()
                .copied()
                .chain((!self.members.contains(&instance.node)).then_some(instance.node))
                .collect(),
            QuorumPolicy::new(self.required_count),
            suspicion_threshold,
            self.epoch,
            true,
            1,
        )
    }
}
