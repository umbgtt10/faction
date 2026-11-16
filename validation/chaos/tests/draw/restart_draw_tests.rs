// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::restart_draw::RestartDraw;
use faction_chaos::model::restart_from::RestartFrom;
use faction_chaos::model::restart_plan::RestartPlan;

fn drawn() -> Vec<RestartPlan> {
    (0..200)
        .flat_map(|seed| RestartDraw::new(seed).restarts(&[1, 3], 150))
        .collect()
}

#[test]
fn restarts_of_one_seed_are_drawn_alike() {
    // Arrange
    let first = RestartDraw::new(6).restarts(&[0, 2], 150);

    // Act
    let second = RestartDraw::new(6).restarts(&[0, 2], 150);

    // Assert
    assert_eq!(first, second);
}

#[test]
fn restarts_start_from_a_genesis_or_a_snapshot() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    assert!(drawn.iter().any(|plan| plan.from == RestartFrom::Genesis));
    assert!(
        drawn
            .iter()
            .any(|plan| matches!(plan.from, RestartFrom::Snapshot { behind } if behind > 0))
    );
}

#[test]
fn restarts_take_down_only_the_nodes_given_within_the_chaos() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    assert!(!drawn.is_empty());
    assert!(
        drawn
            .iter()
            .all(|plan| [1, 3].contains(&plan.peer_id) && plan.at < 150 && plan.down_for <= 30)
    );
}
