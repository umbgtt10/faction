// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::draw::generator::Generator;
use crate::draw::scenario_source::ScenarioSource;
use crate::model::scenario::Scenario;

// The scenario the generator draws from each seed.
pub struct GeneratedScenarios;

impl ScenarioSource for GeneratedScenarios {
    fn scenario(&self, seed: u64) -> Scenario {
        Generator::new(seed).scenario()
    }
}
