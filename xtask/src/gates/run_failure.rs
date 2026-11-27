// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::artifacts::run_log_failure::RunLogFailure;
use serge4rust::gates::gate_failure::GateFailure;
use std::fmt;

// Each way faction's own gates -- the chaos run and the system tests' run --
// fail, said exactly as they have always said it. serge4rust's stage takes
// them as a gate's own failure, in these words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunFailure {
    ChaosFailed { exit_code: Option<i32> },
    JobsFile { path: String, reason: String },
    LogDirectory(RunLogFailure),
    SystemTestFailed { exit_code: Option<i32> },
}

impl From<RunLogFailure> for RunFailure {
    fn from(failure: RunLogFailure) -> Self {
        Self::LogDirectory(failure)
    }
}

impl From<RunFailure> for GateFailure {
    fn from(failure: RunFailure) -> Self {
        Self::Own {
            reason: failure.to_string(),
        }
    }
}

impl fmt::Display for RunFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChaosFailed { exit_code } => {
                write!(formatter, "a chaos run failed, exit code {exit_code:?}")
            }
            Self::JobsFile { path, reason } => {
                write!(
                    formatter,
                    "could not write the job list to {path}: {reason}"
                )
            }
            Self::LogDirectory(failure) => write!(formatter, "{failure}"),
            Self::SystemTestFailed { exit_code } => {
                write!(formatter, "a system test failed, exit code {exit_code:?}")
            }
        }
    }
}
