// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum TimerDelay {
    Minimal,
    Moderate,
    Generous,
}

impl TimerDelay {
    pub fn duration(&self) -> Duration {
        match self {
            TimerDelay::Minimal => Duration::from_millis(10),
            TimerDelay::Moderate => Duration::from_millis(50),
            TimerDelay::Generous => Duration::from_millis(200),
        }
    }
}
