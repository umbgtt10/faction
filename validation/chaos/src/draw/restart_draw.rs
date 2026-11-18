// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::dice::Dice;

use crate::model::restart_from::RestartFrom;
use crate::model::restart_plan::RestartPlan;

const MAX_BEHIND: u64 = 3;
const MAX_DOWN: u64 = 30;
const MAX_RESTARTS: u64 = 2;

// A scenario's crashes: a node down a while, then restarted from its genesis
// or from a snapshot, possibly a stale one.
pub struct RestartDraw {
    dice: Dice,
}

impl RestartDraw {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            dice: Dice::new(seed),
        }
    }

    // Crashes of nodes of `restartable`, within `chaos_rounds`.
    pub fn restarts(&mut self, restartable: &[u64], chaos_rounds: u64) -> Vec<RestartPlan> {
        let count = self.dice.below(MAX_RESTARTS + 1);
        (0..count)
            .map(|_| RestartPlan {
                peer_id: restartable[self.dice.below(restartable.len() as u64) as usize],
                at: self.dice.below(chaos_rounds),
                down_for: self.dice.below(MAX_DOWN + 1),
                from: if self.dice.rolls(50) {
                    RestartFrom::Genesis
                } else {
                    RestartFrom::Snapshot {
                        behind: self.dice.below(MAX_BEHIND) as usize,
                    }
                },
            })
            .collect()
    }
}
