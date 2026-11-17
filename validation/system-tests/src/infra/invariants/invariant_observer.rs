// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::instance::receipt::Receipt;
use std::collections::BTreeSet;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::observer::Observer;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::types::Epoch;
use faction::types::PeerId;

use crate::infra::invariants::breach::Breach;
use crate::infra::invariants::breaches::Breaches;

// A faction observer that checks, after every call its node takes, what the
// node must never do, whatever the network and its timers do: let its epoch
// fall, count a claim from behind, conclude short of the required count,
// show a member it was never told of, or answer, once removed, anything but
// a membership and a probe. A breach goes to `breaches`, and every call on
// to `inner`. A node made at a membership is told of its members; a newcomer
// of those its first view shows, which the membership that made it told it.
pub struct InvariantObserver {
    peer_id: PeerId,
    told: Option<BTreeSet<PeerId>>,
    epoch: Epoch,
    removed: bool,
    breaches: Breaches,
    inner: Box<dyn Observer>,
}

impl InvariantObserver {
    // `told`: the members the node is made with, none for a newcomer; `epoch`:
    // the epoch it is made at.
    #[must_use]
    pub fn new(
        peer_id: PeerId,
        told: Option<Vec<PeerId>>,
        epoch: Epoch,
        breaches: Breaches,
        inner: Box<dyn Observer>,
    ) -> Self {
        Self {
            peer_id,
            told: told.map(|members| members.into_iter().collect()),
            epoch,
            removed: false,
            breaches,
            inner,
        }
    }

    fn answered(&self, command: &Command, result: &ProcessResult) -> bool {
        self.removed
            && !matches!(command, Command::MembershipReceived { .. } | Command::Probe)
            && !matches!(result, ProcessResult::Rejected { .. })
    }

    fn breached(&self, command: &Command, result: &ProcessResult) -> Option<&'static str> {
        let (view, outcomes) = Self::parts(result);
        [
            ("the epoch fell", view.epoch() < self.epoch),
            (
                "a claim from behind counted",
                self.counts_from_behind(command, outcomes),
            ),
            (
                "concluded short of the required count",
                Self::concluded_short(view, outcomes),
            ),
            ("a member it was never told of", self.shows_a_stranger(view)),
            ("a removed node answered", self.answered(command, result)),
        ]
        .into_iter()
        .find(|(_, broken)| *broken)
        .map(|(invariant, _)| invariant)
    }

    fn concluded_short(view: &ClusterView, outcomes: &[Outcome]) -> bool {
        outcomes
            .iter()
            .any(|outcome| matches!(outcome, Outcome::Concluded { .. }))
            && view.collecting_peers().len() < view.required_count()
    }

    // Whether a claim from below the node's epoch was counted: a member's
    // participation or readiness accepted, or the acknowledgement awaited
    // proposed.
    fn counts_from_behind(&self, command: &Command, outcomes: &[Outcome]) -> bool {
        let counted = match command {
            Command::ParticipationObserved { peer_id, epoch } if *epoch < self.epoch => {
                Outcome::ParticipationAccepted { peer_id: *peer_id }
            }
            Command::ReadyObserved { peer_id, epoch } if *epoch < self.epoch => {
                Outcome::ReadyAccepted { peer_id: *peer_id }
            }
            Command::MembershipAcknowledged { peer_id, epoch } if *epoch < self.epoch => {
                Outcome::ProposeAddition { peer_id: *peer_id }
            }
            _ => return false,
        };
        outcomes.contains(&counted)
    }

    // The peers a command tells the node of as members, or may make members.
    fn named(command: &Command) -> Vec<PeerId> {
        match command {
            Command::AdditionCommitted { peer_id, .. }
            | Command::JoinRequested { peer_id }
            | Command::JoinApproved { peer_id } => vec![*peer_id],
            Command::MembershipReceived { members, .. } => members.clone(),
            _ => Vec::new(),
        }
    }

    fn parts(result: &ProcessResult) -> (&ClusterView, &[Outcome]) {
        match result {
            ProcessResult::Accepted {
                cluster_view,
                outcomes,
                ..
            } => (cluster_view, outcomes),
            ProcessResult::Rejected { cluster_view, .. }
            | ProcessResult::Probed { cluster_view, .. } => (cluster_view, &[]),
        }
    }

    fn shows_a_stranger(&self, view: &ClusterView) -> bool {
        self.told.as_ref().is_some_and(|told| {
            view.members()
                .as_slice()
                .iter()
                .any(|member| !told.contains(member))
        })
    }

    fn tell(&mut self, command: &Command, view: &ClusterView) {
        let told = self
            .told
            .get_or_insert_with(|| view.members().as_slice().iter().copied().collect());
        told.extend(Self::named(command));
    }
}

impl Observer for InvariantObserver {
    fn observe(&mut self, command: &Command, receipt: &Receipt) {
        let result = &receipt.output;
        let (view, _) = Self::parts(result);
        self.tell(command, view);
        if let Some(invariant) = self.breached(command, result) {
            self.breaches.report(Breach {
                peer_id: self.peer_id,
                invariant,
                command: command.clone(),
            });
        }
        self.epoch = view.epoch();
        self.removed = view.peer_state() == PeerState::Removed;
        self.inner.observe(command, receipt);
    }
}
