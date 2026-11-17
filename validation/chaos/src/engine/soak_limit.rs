// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

// How long a soak plays: so many scenarios, or for so long.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoakLimit {
    Scenarios(u64),
    For(Duration),
}

impl SoakLimit {
    // Whether a soak that has played `scenarios` in `elapsed` is done.
    #[must_use]
    pub fn reached(&self, scenarios: u64, elapsed: Duration) -> bool {
        match self {
            Self::Scenarios(limit) => scenarios >= *limit,
            Self::For(limit) => elapsed >= *limit,
        }
    }
}
