// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::dice::Dice;

use crate::model::change::Change;
use crate::model::planned_change::PlannedChange;

const BURST_PERCENT: u8 = 50;
const COUNT_CHANGE_PERCENT: u8 = 30;
const MAX_NEWCOMERS: u64 = 2;
const MAX_REMOVALS: usize = 2;
const READDITION_PERCENT: u8 = 50;

// A scenario's changes, drawn so that the committed log stays valid in
// whatever order they commit: genesis members removed never below the
// required count -- the initial one, or the one a count change sets --
// newcomers numbered on from the genesis peers, and a removed member taken
// back only after its removal. In half the scenarios every change comes due
// in one round, a burst, so that several are in flight at once; in the
// others each in a round of its own.
pub struct ChangeDraw {
    dice: Dice,
    burst: Option<u64>,
}

impl ChangeDraw {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            dice: Dice::new(seed),
            burst: None,
        }
    }

    // The changes to a cluster of `nodes` requiring `required_count`, each due
    // within `chaos_rounds`.
    pub fn changes(
        &mut self,
        nodes: usize,
        required_count: usize,
        chaos_rounds: u64,
    ) -> Vec<PlannedChange> {
        self.burst = self
            .dice
            .rolls(BURST_PERCENT)
            .then(|| self.dice.below(chaos_rounds));
        let removed = self.removed(nodes, required_count);
        let newcomers = self.dice.below(MAX_NEWCOMERS + 1);
        let mut changes: Vec<PlannedChange> = (0..newcomers)
            .map(|offset| {
                let peer_id = nodes as u64 + offset;
                self.planned(Change::Addition { peer_id }, None, chaos_rounds)
            })
            .collect();
        for peer_id in removed.iter().copied() {
            changes.push(self.planned(Change::Removal { peer_id }, None, chaos_rounds));
            if self.dice.rolls(READDITION_PERCENT) {
                let after = Some(changes.len() - 1);
                changes.push(self.planned(Change::Addition { peer_id }, after, chaos_rounds));
            }
        }
        if self.dice.rolls(COUNT_CHANGE_PERCENT) {
            let floor = (nodes - removed.len()) as u64;
            let required_count = 1 + self.dice.below(floor) as usize;
            changes.push(self.planned(
                Change::RequiredCount { required_count },
                None,
                chaos_rounds,
            ));
        }
        changes
    }

    fn planned(
        &mut self,
        change: Change,
        after: Option<usize>,
        chaos_rounds: u64,
    ) -> PlannedChange {
        PlannedChange {
            change,
            due: self.burst.unwrap_or_else(|| self.dice.below(chaos_rounds)),
            after,
        }
    }

    // Genesis members to remove, never so many that the members would fall
    // below the required count.
    fn removed(&mut self, nodes: usize, required_count: usize) -> Vec<u64> {
        let room = (nodes - required_count).min(MAX_REMOVALS) as u64;
        let count = self.dice.below(room + 1);
        let mut candidates: Vec<u64> = (0..nodes as u64).collect();
        (0..count)
            .map(|_| {
                let index = self.dice.below(candidates.len() as u64) as usize;
                candidates.remove(index)
            })
            .collect()
    }
}
