// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::fs::read_to_string;
use std::fs::write;
use std::io::ErrorKind;
use std::path::PathBuf;

use serde_json::from_str;
use serde_json::to_string_pretty;

use crate::engine::regression::Regression;
use crate::engine::regressions_error::RegressionsError;

// The scenarios soaks found failing, kept as JSON in one file: a soak records
// each it shrinks, and the single run plays every one, as proptest replays
// its regressions. A file not yet written holds none.
pub struct Regressions {
    path: PathBuf,
}

impl Regressions {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Vec<Regression>, RegressionsError> {
        match read_to_string(&self.path) {
            Ok(text) => from_str(&text).map_err(|error| self.error(&error.to_string())),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(self.error(&error.to_string())),
        }
    }

    // `regression` added to those the file holds.
    pub fn record(&self, regression: Regression) -> Result<(), RegressionsError> {
        let mut regressions = self.load()?;
        regressions.push(regression);
        let text =
            to_string_pretty(&regressions).map_err(|error| self.error(&error.to_string()))?;
        write(&self.path, text + "\n").map_err(|error| self.error(&error.to_string()))
    }

    fn error(&self, reason: &str) -> RegressionsError {
        RegressionsError {
            path: self.path.clone(),
            reason: String::from(reason),
        }
    }
}
