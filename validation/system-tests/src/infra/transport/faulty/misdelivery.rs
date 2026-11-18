// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use crate::infra::dice::Dice;

// Datagrams reaching a node they were not sent to, as when two nodes answer
// for one address: now and then, a message goes to another peer this node
// has sent to, drawn from a seed of its own.
pub struct Misdelivery {
    known: BTreeSet<u64>,
    dice: Dice,
}

impl Misdelivery {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            known: BTreeSet::new(),
            dice: Dice::new(seed),
        }
    }

    // The peer a message to `to` reaches: `to`, or, at `percent`, another it
    // has sent to before, if there is one.
    pub fn addressee(&mut self, to: u64, percent: u8) -> u64 {
        self.known.insert(to);
        if !self.dice.rolls(percent) {
            return to;
        }
        let others: Vec<u64> = self
            .known
            .iter()
            .copied()
            .filter(|peer| *peer != to)
            .collect();
        let count = u64::try_from(others.len()).unwrap_or(u64::MAX);
        if count == 0 {
            return to;
        }
        let index = usize::try_from(self.dice.below(count)).unwrap_or(0);
        others[index]
    }
}
