// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Serialize;

use crate::model::scenario::Scenario;

// A scenario a soak found failing, shrunk: the seed it came from, how it
// failed, and the scenario whole, which the single run plays from then on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Regression {
    pub seed: u64,
    pub failure: String,
    pub scenario: Scenario,
}
