// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// A seeded stream of draws: the same seed draws the same faults, run after
// run. An even seed draws as its odd neighbour, since a xorshift's state is
// never 0.
pub struct Dice {
    state: u64,
}

impl Dice {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    // A draw below `bound`, which is above 0.
    pub fn below(&mut self, bound: u64) -> u64 {
        self.draw() % bound
    }

    pub fn draw(&mut self) -> u64 {
        let mut state = self.state;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.state = state;
        state
    }

    // Whether a roll falls within `percent` of a hundred: never at 0, always
    // at 100 or more, and neither draws.
    pub fn rolls(&mut self, percent: u8) -> bool {
        if percent == 0 {
            return false;
        }
        if percent >= 100 {
            return true;
        }
        self.below(100) < u64::from(percent)
    }
}
