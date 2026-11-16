// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::model::scenario::Scenario;
use crate::oracle::failure::Failure;

// The smallest scenario a shrink found still failing, and how it fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shrunk {
    pub scenario: Scenario,
    pub failure: Failure,
}
