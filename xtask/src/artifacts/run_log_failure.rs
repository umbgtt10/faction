// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::fmt;
use std::path::PathBuf;

// What stopped a run's log directory being made or the old runs being cleared
// away, with the path it happened to and the reason the system gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunLogFailure {
    Create { path: PathBuf, reason: String },
    Read { path: PathBuf, reason: String },
    Remove { path: PathBuf, reason: String },
}

impl fmt::Display for RunLogFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create { path, reason } => {
                write!(formatter, "failed to create {}: {reason}", path.display())
            }
            Self::Read { path, reason } => {
                write!(formatter, "failed to read {}: {reason}", path.display())
            }
            Self::Remove { path, reason } => {
                write!(formatter, "failed to remove {}: {reason}", path.display())
            }
        }
    }
}
