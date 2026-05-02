// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
