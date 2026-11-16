// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::generator::Generator;
use faction_chaos::model::change::Change;
use faction_chaos::model::fault::Fault;
use faction_chaos::model::scenario::Scenario;

fn drawn() -> Vec<Scenario> {
    (0..300)
        .map(|seed| Generator::new(seed).scenario())
        .collect()
}

fn removed(scenario: &Scenario) -> Vec<u64> {
    scenario
        .changes
        .iter()
        .filter_map(|planned| match planned.change {
            Change::Removal { peer_id } => Some(peer_id),
            _ => None,
        })
        .collect()
}

#[test]
fn scenario_draws_a_cluster_of_three_to_six_that_its_count_fits() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    assert!(
        drawn
            .iter()
            .all(|scenario| (3..=6).contains(&scenario.nodes)
                && (1..=scenario.nodes).contains(&scenario.required_count)
                && (1..=3).contains(&scenario.suspicion_threshold))
    );
    assert!(drawn.iter().any(|scenario| scenario.nodes == 3));
    assert!(drawn.iter().any(|scenario| scenario.nodes == 6));
}

#[test]
fn scenario_of_another_seed_is_another() {
    // Arrange
    let first = Generator::new(1).scenario();

    // Act
    let second = Generator::new(2).scenario();

    // Assert
    assert_ne!(first, second);
}

#[test]
fn scenario_of_one_seed_is_drawn_alike() {
    // Arrange
    let first = Generator::new(42).scenario();

    // Act
    let second = Generator::new(42).scenario();

    // Assert
    assert_eq!(first, second);
    assert_eq!(first.seed, 42);
}

#[test]
fn scenario_restarts_and_kills_timers_only_of_members_never_removed() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    for scenario in drawn {
        let removed = removed(&scenario);
        let stable = |peer_id: u64| peer_id < scenario.nodes as u64 && !removed.contains(&peer_id);
        assert!(scenario.restarts.iter().all(|plan| stable(plan.peer_id)));
        assert!(
            scenario
                .faults
                .iter()
                .filter(|window| matches!(window.fault, Fault::Timer(policy) if policy.dead))
                .all(|window| stable(window.peer_id))
        );
    }
}

#[test]
fn scenario_spreads_its_faults_over_every_peer_it_runs() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    assert!(drawn.iter().any(|scenario| {
        scenario
            .faults
            .iter()
            .any(|window| window.peer_id >= scenario.nodes as u64)
    }));
    assert!(
        drawn
            .iter()
            .any(|scenario| scenario.duplicates > 0 && scenario.delivery_lag > 0)
    );
}
