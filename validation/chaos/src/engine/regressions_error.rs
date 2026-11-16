// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::path::PathBuf;

// The regressions file could not be read, understood or written: which file,
// and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegressionsError {
    pub path: PathBuf,
    pub reason: String,
}

impl Display for RegressionsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}: {}", self.path.display(), self.reason)
    }
}

impl Error for RegressionsError {}
