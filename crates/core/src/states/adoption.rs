// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::command::Command;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::quorum_policy::QuorumPolicy;
use crate::stage::Stage;
use crate::states::initial::Initial;
use crate::states::removed::Removed;
use crate::types::Epoch;
use crate::types::PeerId;

// A membership handed to the node above its epoch, which it takes whatever its
// stage (§16): its members, its required count and its epoch. Naming the node,
// a stage adopts it as a membership decision; not naming it, it removes the
// node; and it is a removed node's way back.
pub struct Adoption {
    members: Members,
    quorum_policy: QuorumPolicy,
    epoch: Epoch,
}

impl Adoption {
    #[must_use]
    pub fn of(command: &Command) -> Option<Self> {
        match command {
            Command::MembershipReceived {
                members,
                required_count,
                epoch,
            } => Some(Self {
                members: Members::new(members.clone()),
                quorum_policy: QuorumPolicy::new(*required_count),
                epoch: *epoch,
            }),
            _ => None,
        }
    }

    // The node `peer_id` removed, adopting the membership, if it does not
    // name the node; none if it does.
    #[must_use]
    pub fn removal(self, peer_id: PeerId) -> Option<(Vec<Outcome>, Stage)> {
        (!self.members.is_member(peer_id)).then(|| {
            (
                vec![
                    Outcome::MembershipAdopted { epoch: self.epoch },
                    Outcome::MemberRemoved { peer_id },
                ],
                Stage::Removed(Removed::new(peer_id, self.members, self.quorum_policy)),
            )
        })
    }

    // A removed node's answer: back to `Initial`, a fresh node holding the
    // membership, if it names `peer_id`; still `Removed`, holding it,
    // otherwise.
    #[must_use]
    pub fn taken_back(self, peer_id: PeerId) -> (Vec<Outcome>, Stage) {
        let adopted = vec![Outcome::MembershipAdopted { epoch: self.epoch }];
        if self.members.is_member(peer_id) {
            (
                adopted,
                Stage::Initial(Initial::new(self.members, self.quorum_policy)),
            )
        } else {
            (
                adopted,
                Stage::Removed(Removed::new(peer_id, self.members, self.quorum_policy)),
            )
        }
    }
}
