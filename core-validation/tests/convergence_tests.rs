// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::conclusion::Conclusion;
use faction::peer_state::PeerState;
use faction_core_validation::cluster_simulation::ClusterSimulation;

extern crate alloc;

#[test]
fn check_after_convergence_returns_no_violation() {
    // Arrange
    let mut sim = ClusterSimulation::new(5, 4);
    for peer in 1..5 {
        sim.inject_participation(peer);
    }
    for peer in 0..5 {
        sim.complete_local(peer);
    }

    // Act
    let violations = sim.check();

    // Assert
    assert!(violations.is_empty(), "{violations:?}");
    assert!(sim.all_exited_with(Conclusion::Bootstrapped));
}

#[test]
fn duplicate_signals_dont_disrupt_convergence() {
    // Arrange
    let mut sim = ClusterSimulation::new(5, 4);

    // Act
    for peer in 1..5 {
        sim.inject_participation(peer);
        sim.inject_participation(peer);
    }
    for peer in 0..5 {
        sim.complete_local(peer);
    }

    // Assert
    assert!(sim.pending_count() == 0);
    assert!(sim.all_exited_with(Conclusion::Bootstrapped));
}

#[test]
fn five_nodes_converge_on_quorum() {
    // Arrange
    let mut sim = ClusterSimulation::new(5, 4);

    // Act
    for peer in 1..5 {
        sim.inject_participation(peer);
    }
    for peer in 0..5 {
        sim.complete_local(peer);
    }

    // Assert
    assert!(sim.all_exited_with(Conclusion::Bootstrapped));
}

#[test]
fn not_enough_signals_records_deadline_miss() {
    // Arrange
    let mut sim = ClusterSimulation::new(5, 4);
    sim.inject_participation(1);
    sim.complete_local(0);
    sim.inject_ready(1);

    // Act
    for peer in 0..5 {
        sim.expire_deadline(peer);
    }

    // Assert -- a missed deadline is recorded, not terminal: every node
    // reports TimedOut yet none has concluded, so they stay receptive.
    for peer in 0..5 {
        let view = sim.cluster_view(peer);
        assert_eq!(view.peer_state(), PeerState::TimedOut);
        assert!(!view.is_concluded());
        assert!(view.deadline_missed());
    }
}
