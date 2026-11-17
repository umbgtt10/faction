// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::time::Duration;

// A poll that gave up: what it waited for, and how long it waited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollExpired {
    pub waiting_for: String,
    pub deadline: Duration,
}

impl Display for PollExpired {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "gave up after {:?} waiting for {}",
            self.deadline, self.waiting_for
        )
    }
}

impl Error for PollExpired {}
