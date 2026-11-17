// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::model::scenario::Scenario;

// Where a soak takes the scenario of each seed from.
pub trait ScenarioSource {
    fn scenario(&self, seed: u64) -> Scenario;
}
