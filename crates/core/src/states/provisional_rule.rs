// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::command::Command;
use crate::outcome::Outcome;
use crate::types::PeerId;

// What a provisional membership decides (§16): a node holding one sends a
// peer behind no membership, holding none the layer above committed; and its
// membership stops being provisional, once and for all, when it applies the
// commit of its own addition, adopts a membership, or is removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProvisionalRule {
    peer_id: PeerId,
    provisional: bool,
}

impl ProvisionalRule {
    #[must_use]
    pub const fn new(peer_id: PeerId, provisional: bool) -> Self {
        Self {
            peer_id,
            provisional,
        }
    }

    // Whether the node's membership is still provisional once `command` is
    // answered with `outcomes`.
    #[must_use]
    pub fn after(&self, command: &Command, outcomes: &[Outcome]) -> bool {
        self.provisional && !self.ends(command, outcomes)
    }

    // The outcomes as the node gives them: at a provisional node, without
    // the membership a claim from behind would send its peer.
    #[must_use]
    pub fn answered(&self, outcomes: Vec<Outcome>) -> Vec<Outcome> {
        if !self.provisional {
            return outcomes;
        }
        let refused: Vec<PeerId> = outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                Outcome::StaleEpochIgnored { peer_id, .. } => Some(*peer_id),
                _ => None,
            })
            .collect();
        outcomes
            .into_iter()
            .filter(|outcome| {
                !matches!(outcome, Outcome::SendMembership { peer_id } if refused.contains(peer_id))
            })
            .collect()
    }

    // Whether the answer ends a provisional membership: a membership adopted,
    // or a commit of the node's own addition or removal applied.
    fn ends(&self, command: &Command, outcomes: &[Outcome]) -> bool {
        let own = matches!(
            *command,
            Command::AdditionCommitted { peer_id, .. } | Command::RemovalCommitted { peer_id, .. }
                if peer_id == self.peer_id
        );
        outcomes
            .iter()
            .any(|outcome| matches!(outcome, Outcome::MembershipAdopted { .. }))
            || own
                && !outcomes
                    .iter()
                    .any(|outcome| matches!(outcome, Outcome::CommitIgnored { .. }))
    }
}
