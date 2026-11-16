// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::fault_draw::FaultDraw;
use faction_chaos::model::fault::Fault;
use faction_chaos::model::fault_window::FaultWindow;

fn drawn() -> Vec<FaultWindow> {
    (0..300)
        .flat_map(|seed| FaultDraw::new(seed).windows(&[0, 1, 2, 3, 4], &[0, 1], 200))
        .collect()
}

#[test]
fn windows_cover_each_layer_of_a_node() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    assert!(
        drawn
            .iter()
            .any(|window| matches!(window.fault, Fault::Transport(_)))
    );
    assert!(
        drawn
            .iter()
            .any(|window| matches!(window.fault, Fault::Timer(_)))
    );
    assert!(
        drawn
            .iter()
            .any(|window| matches!(window.fault, Fault::Clock(_)))
    );
}

#[test]
fn windows_kill_a_timer_side_only_on_a_node_a_restart_heals() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    let dead: Vec<u64> = drawn
        .iter()
        .filter(|window| matches!(window.fault, Fault::Timer(policy) if policy.dead))
        .map(|window| window.peer_id)
        .collect();
    assert!(!dead.is_empty());
    assert!(dead.iter().all(|peer_id| [0, 1].contains(peer_id)));
}

#[test]
fn windows_of_one_seed_are_drawn_alike() {
    // Arrange
    let first = FaultDraw::new(4).windows(&[0, 1, 2], &[0], 150);

    // Act
    let second = FaultDraw::new(4).windows(&[0, 1, 2], &[0], 150);

    // Assert
    assert_eq!(first, second);
}

#[test]
fn windows_open_within_the_chaos_and_last_ten_rounds_or_more() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    assert!(
        drawn
            .iter()
            .all(|window| window.from < 200 && window.until >= window.from + 10)
    );
}
