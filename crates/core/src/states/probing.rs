// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::instance::peer_id::PeerId;
use crate::members::Members;
use crate::outcome::Outcome;

// Whom a concluded node probes, one period at a time: the member after the one
// it probed last, in the members' order, past the last back to the first,
// itself skipped; and whom it asks to probe a member that missed.
#[derive(Debug, Clone, Copy)]
pub struct Probing<P: PeerId = u64> {
    last_probed: Option<P>,
}

impl<P: PeerId> Probing<P> {
    // Every member not suspected but the node and the missed one, from the
    // member after the missed one, past the last back to the first.
    #[must_use]
    pub fn helpers(members: &Members<P>, missed: P, local: P, suspected: &[P]) -> Vec<P> {
        Self::after(members.as_slice(), Some(missed))
            .filter(|member| *member != local && *member != missed && !suspected.contains(member))
            .collect()
    }

    #[must_use]
    pub fn last_probed(&self) -> Option<P> {
        self.last_probed
    }

    // Whom it probes among `members` alone: if the member probed last is not
    // one, the node has probed no one.
    #[must_use]
    pub fn of_members(&self, members: &Members<P>) -> Self {
        Self {
            last_probed: self.last_probed.filter(|last| members.is_member(*last)),
        }
    }

    #[must_use]
    pub fn probe(&self, members: &Members<P>, local: P) -> (Vec<Outcome<P>>, Self) {
        match Self::after(members.as_slice(), self.last_probed).find(|member| *member != local) {
            Some(peer_id) => (
                vec![Outcome::SendProbe { peer_id }],
                Self {
                    last_probed: Some(peer_id),
                },
            ),
            None => (vec![Outcome::NothingToProbe], *self),
        }
    }

    // Every member once, starting after `from`: from the first, with no
    // `from`, or one that is no member.
    fn after(members: &[P], from: Option<P>) -> impl Iterator<Item = P> + '_ {
        let start = from
            .and_then(|from| members.iter().position(|member| *member == from))
            .map_or(0, |position| position + 1);
        members
            .iter()
            .copied()
            .cycle()
            .skip(start)
            .take(members.len())
    }
}

impl<P: PeerId> Default for Probing<P> {
    fn default() -> Self {
        Self { last_probed: None }
    }
}
