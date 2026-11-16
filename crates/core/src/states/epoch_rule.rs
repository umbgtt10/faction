// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::command::Command;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::types::Epoch;
use crate::types::PeerId;

// What the node's epoch decides (§16): a commit or a membership it holds
// already is ignored, a claim from behind is refused, and a commit or a
// membership accepted above it moves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochRule {
    epoch: Epoch,
}

impl EpochRule {
    #[must_use]
    pub const fn new(epoch: Epoch) -> Self {
        Self { epoch }
    }

    // The node's epoch once the command is accepted.
    #[must_use]
    pub fn after(&self, command: &Command) -> Epoch {
        match *command {
            Command::AdditionCommitted { epoch, .. }
            | Command::RemovalCommitted { epoch, .. }
            | Command::RequiredCountChanged { epoch, .. }
            | Command::MembershipReceived { epoch, .. } => self.epoch.max(epoch),
            _ => self.epoch,
        }
    }

    // What a command the node holds already is answered: a commit at an epoch
    // of one or more, at or below the node's, or a membership at or below it.
    #[must_use]
    pub fn held(&self, command: &Command) -> Option<Outcome> {
        match *command {
            Command::AdditionCommitted { epoch, .. }
            | Command::RemovalCommitted { epoch, .. }
            | Command::RequiredCountChanged { epoch, .. }
                if (1..=self.epoch).contains(&epoch) =>
            {
                Some(Outcome::CommitIgnored { epoch })
            }
            Command::MembershipReceived { epoch, .. } if epoch <= self.epoch => {
                Some(Outcome::MembershipIgnored { epoch })
            }
            _ => None,
        }
    }

    // What a claim of `peer_id` at `epoch` is answered from behind: refused,
    // and the peer sent the membership.
    #[must_use]
    pub fn refused(&self, peer_id: PeerId, epoch: Epoch) -> Option<Vec<Outcome>> {
        (epoch < self.epoch).then(|| {
            vec![
                Outcome::StaleEpochIgnored { peer_id, epoch },
                Outcome::SendMembership { peer_id },
            ]
        })
    }

    // What a member's participation or readiness is answered from behind.
    #[must_use]
    pub fn signal_refused(&self, command: &Command) -> Option<Vec<Outcome>> {
        match *command {
            Command::ParticipationObserved { peer_id, epoch }
            | Command::ReadyObserved { peer_id, epoch } => self.refused(peer_id, epoch),
            _ => None,
        }
    }

    // What a signal the node does not count is answered: a non-member's
    // participation or readiness ignored, a member's from behind refused.
    #[must_use]
    pub fn uncounted(&self, command: &Command, members: &Members) -> Option<Vec<Outcome>> {
        members
            .non_member_observed(command)
            .map(|peer_id| vec![Outcome::NonMemberIgnored { peer_id }])
            .or_else(|| self.signal_refused(command))
    }
}
