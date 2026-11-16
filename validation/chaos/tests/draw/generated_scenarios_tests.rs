// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::generated_scenarios::GeneratedScenarios;
use faction_chaos::draw::generator::Generator;
use faction_chaos::draw::scenario_source::ScenarioSource;

#[test]
fn scenario_of_a_seed_is_the_one_the_generator_draws() {
    // Arrange
    let source = GeneratedScenarios;

    // Act
    let scenario = source.scenario(17);

    // Assert
    assert_eq!(scenario, Generator::new(17).scenario());
}
