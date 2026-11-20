// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::instance::peer_id::PeerId;
use crate::outcome::Outcome;
use crate::reconfiguration::Reconfiguration;
use crate::stage::Stage;
use crate::states::bootstrapped::Bootstrapped;
use crate::states::coordinating::Coordinating;
use crate::states::epoch_rule::EpochRule;
use crate::states::reconfiguring::Reconfiguring;
use crate::types::Epoch;

// A coordinator's changes in flight, in the order requested (§2, §4): no two
// name one peer, and each ends on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InFlight<P: PeerId = u64> {
    changes: Vec<Reconfiguration<P>>,
}

impl<P: PeerId> InFlight<P> {
    // Every change given up for a membership past them (§16): one
    // abandonment each, in the order requested.
    #[must_use]
    pub fn abandoned(&self) -> Vec<Outcome<P>> {
        self.changes
            .iter()
            .map(|change| Outcome::ChangeAbandoned {
                reconfiguration: *change,
            })
            .collect()
    }

    // The changes with the addition of `peer_id`, requested, acknowledged.
    #[must_use]
    pub fn acknowledged(&self, peer_id: P) -> Self {
        Self {
            changes: self
                .changes
                .iter()
                .map(|change| {
                    if *change == (Reconfiguration::AdditionRequested { peer_id }) {
                        Reconfiguration::AdditionAcknowledged { peer_id }
                    } else {
                        *change
                    }
                })
                .collect(),
        }
    }

    // The awaited acknowledgement answered: refused from behind, proposed
    // otherwise, the addition acknowledged.
    #[must_use]
    pub fn acknowledgement(
        &self,
        peer_id: P,
        claimed: Epoch,
        epoch: Epoch,
    ) -> (Vec<Outcome<P>>, Self) {
        match EpochRule::new(epoch).refused(peer_id, claimed) {
            Some(refused) => (refused, self.clone()),
            None => (
                vec![Outcome::ProposeAddition { peer_id }],
                self.acknowledged(peer_id),
            ),
        }
    }

    // Whether the addition of `peer_id` is in flight, requested or
    // acknowledged.
    #[must_use]
    pub fn adds(&self, peer_id: P) -> bool {
        matches!(
            self.named(peer_id),
            Some(
                Reconfiguration::AdditionRequested { .. }
                    | Reconfiguration::AdditionAcknowledged { .. }
            )
        )
    }

    // What a command answers of the changes in flight, of `concluded`: its
    // outcomes and the changes left -- an acknowledgement, an abort, a
    // decline -- or none, when the concluded node answers it alone.
    #[must_use]
    pub fn answered(
        &self,
        command: &Command<P>,
        concluded: &Bootstrapped<P>,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome<P>>, Self)> {
        match *command {
            Command::MembershipAcknowledged {
                peer_id,
                epoch: claimed,
            } if self.named(peer_id) == Some(Reconfiguration::AdditionRequested { peer_id }) => {
                Some(self.acknowledgement(peer_id, claimed, epoch))
            }
            Command::MembershipAcknowledged { peer_id, .. } => Some((
                vec![Outcome::AcknowledgementIgnored { peer_id }],
                self.clone(),
            )),
            Command::AdditionAborted { peer_id } if self.adds(peer_id) => Some((
                vec![Outcome::AdditionWithdrawn { peer_id }],
                self.without(peer_id),
            )),
            Command::RemovalAborted { peer_id } if self.removes(peer_id) => Some((
                vec![Outcome::RemovalWithdrawn { peer_id }],
                self.without(peer_id),
            )),
            Command::AdditionAborted { peer_id } | Command::RemovalAborted { peer_id } => {
                Some((vec![Outcome::AbortIgnored { peer_id }], self.clone()))
            }
            Command::ProposalDeclined { peer_id, .. } => Some(self.declined(peer_id, concluded)),
            _ => None,
        }
    }

    // A declined proposal decided again on the node's present view (§17): an
    // addition proposed again, a removal proposed again while `concluded` can
    // spare its member, the other removals in flight counted, and given up
    // once it cannot; a decline naming no proposal in flight ignored.
    #[must_use]
    pub fn declined(&self, peer_id: P, concluded: &Bootstrapped<P>) -> (Vec<Outcome<P>>, Self) {
        match self.named(peer_id) {
            Some(Reconfiguration::AdditionAcknowledged { .. }) => {
                (vec![Outcome::ProposeAddition { peer_id }], self.clone())
            }
            Some(Reconfiguration::RemovalRequested { leaves, .. })
                if !concluded.can_spare(peer_id, &self.removals(), leaves) =>
            {
                (
                    vec![Outcome::RemovalRefused { peer_id }],
                    self.without(peer_id),
                )
            }
            Some(Reconfiguration::RemovalRequested { .. }) => {
                (vec![Outcome::ProposeRemoval { peer_id }], self.clone())
            }
            _ => (vec![Outcome::DeclineIgnored { peer_id }], self.clone()),
        }
    }

    // The peer of the change a commit ends: its addition or its removal in
    // flight.
    #[must_use]
    pub fn ended_by(&self, command: &Command<P>) -> Option<P> {
        match *command {
            Command::AdditionCommitted { peer_id, .. } if self.adds(peer_id) => Some(peer_id),
            Command::RemovalCommitted { peer_id, .. } if self.removes(peer_id) => Some(peer_id),
            _ => None,
        }
    }

    // The change in flight naming `peer_id`, if one is: no two do.
    #[must_use]
    pub fn named(&self, peer_id: P) -> Option<Reconfiguration<P>> {
        self.changes.iter().copied().find(|change| match *change {
            Reconfiguration::AdditionRequested { peer_id: named }
            | Reconfiguration::AdditionAcknowledged { peer_id: named }
            | Reconfiguration::RemovalRequested { peer_id: named, .. } => named == peer_id,
        })
    }

    // Every peer whose removal is in flight, in the order requested.
    #[must_use]
    pub fn removals(&self) -> Vec<P> {
        self.changes
            .iter()
            .filter_map(|change| match change {
                Reconfiguration::RemovalRequested { peer_id, .. } => Some(*peer_id),
                _ => None,
            })
            .collect()
    }

    // Whether the removal of `peer_id` is in flight.
    #[must_use]
    pub fn removes(&self, peer_id: P) -> bool {
        matches!(
            self.named(peer_id),
            Some(Reconfiguration::RemovalRequested { .. })
        )
    }

    // The view a builder makes, showing the changes in flight.
    #[must_use]
    pub fn shown(&self, builder: ClusterViewBuilder<P>) -> ClusterViewBuilder<P> {
        builder.with_reconfigurations(self.changes.clone())
    }

    // The stage the changes make of the concluded node (§2): none Bootstrapped,
    // fewer than the bound Coordinating, as many Reconfiguring.
    #[must_use]
    pub fn stage(self, concluded: Bootstrapped<P>, config: &Config<P>) -> Stage<P> {
        if self.changes.is_empty() {
            Stage::Bootstrapped(concluded)
        } else if self.changes.len() < config.changes_in_flight() {
            Stage::Coordinating(Coordinating::new(Reconfiguring::new(concluded, self)))
        } else {
            Stage::Reconfiguring(Reconfiguring::new(concluded, self))
        }
    }

    // The changes with `change` requested last.
    #[must_use]
    pub fn with(&self, change: Reconfiguration<P>) -> Self {
        Self {
            changes: self.changes.iter().copied().chain([change]).collect(),
        }
    }

    // The changes but the one naming `peer_id`, which has ended.
    #[must_use]
    pub fn without(&self, peer_id: P) -> Self {
        let ended = self.named(peer_id);
        Self {
            changes: self
                .changes
                .iter()
                .copied()
                .filter(|change| Some(*change) != ended)
                .collect(),
        }
    }
}

impl<P: PeerId> Default for InFlight<P> {
    fn default() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
}
