// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;

use crate::traits::monotonic_clock::MonotonicClock;
use crate::traits::wall_clock::WallClock;

// A wall clock reading a monotonic one on from a fixed origin: a run on a
// simulated clock stamps its logs with the time it simulates, alike every
// time it replays.
pub struct SimulatedWallClock {
    clock: Arc<dyn MonotonicClock>,
    origin: DateTime<Utc>,
}

impl SimulatedWallClock {
    #[must_use]
    pub fn new(clock: Arc<dyn MonotonicClock>, origin: DateTime<Utc>) -> Self {
        Self { clock, origin }
    }
}

impl WallClock for SimulatedWallClock {
    fn now(&self) -> DateTime<Utc> {
        self.origin + TimeDelta::from_std(self.clock.now()).unwrap_or(TimeDelta::MAX)
    }
}
