// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Mutex;
use std::time::Duration;

use crate::traits::monotonic_clock::MonotonicClock;
use crate::traits::pause::Pause;

// Time that passes only when it is paused for, or advanced: every reading,
// and so every timer's firing, fixed by the run that asks for it -- what
// makes a run on it replay exactly.
pub struct SimulatedClock {
    now: Mutex<Duration>,
}

impl SimulatedClock {
    #[must_use]
    pub fn at(now: Duration) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    pub fn advance(&self, by: Duration) {
        *self.now.lock().unwrap() += by;
    }
}

impl MonotonicClock for SimulatedClock {
    fn now(&self) -> Duration {
        *self.now.lock().unwrap()
    }
}

impl Pause for SimulatedClock {
    fn pause(&self, duration: Duration) {
        self.advance(duration);
    }
}
