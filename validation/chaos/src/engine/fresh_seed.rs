// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use faction_system_tests::traits::wall_clock::WallClock;

// A soak's first seed when it is given none: the wall clock's nanoseconds
// since the epoch. Soaks started a second apart begin a billion seeds apart,
// more than any soak plays; a count of seconds began two soaks an hour apart
// some 3,600 seeds apart, and the second replayed nearly all the first had.
pub struct FreshSeed {
    clock: Arc<dyn WallClock>,
}

impl FreshSeed {
    #[must_use]
    pub fn new(clock: Arc<dyn WallClock>) -> Self {
        Self { clock }
    }

    #[must_use]
    pub fn seed(&self) -> u64 {
        self.clock.now().nanoseconds()
    }
}
