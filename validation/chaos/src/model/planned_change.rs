// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Serialize;

use crate::model::change::Change;

// A change as a scenario plans it: the round it is due from, and the change
// that must commit before it, by its place in the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedChange {
    pub change: Change,
    pub due: u64,
    pub after: Option<usize>,
}
