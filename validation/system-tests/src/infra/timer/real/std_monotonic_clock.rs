// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use crate::traits::monotonic_clock::MonotonicClock;

pub struct StdMonotonicClock {
    origin: OnceLock<Instant>,
}

impl StdMonotonicClock {
    #[must_use]
    pub fn new() -> Self {
        Self {
            origin: OnceLock::new(),
        }
    }
}

impl Default for StdMonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MonotonicClock for StdMonotonicClock {
    fn now(&self) -> Duration {
        self.origin.get_or_init(Instant::now).elapsed()
    }
}
