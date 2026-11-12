// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::conclusion::Conclusion;
use faction::peer_state::PeerState;
use faction_core_validation::cluster_simulation::ClusterSimulation;

#[test]
fn all_exited_with_before_any_signal_returns_false() {
    // Arrange
    let mut sim = ClusterSimulation::new(3, 2);

    // Act
    let all_exited = sim.all_exited_with(Conclusion::Bootstrapped);

    // Assert
    assert!(!all_exited);
}

#[test]
fn cluster_view_reports_the_named_peer() {
    // Arrange
    let mut sim = ClusterSimulation::new(3, 3);
    sim.complete_local(0);

    // Act
    let completed = sim.cluster_view(0);
    let pinging = sim.cluster_view(1);

    // Assert
    assert_eq!(completed.peer_state(), PeerState::Collecting);
    assert_eq!(pinging.peer_state(), PeerState::Pinging);
}

#[test]
fn inject_participation_is_counted_by_every_node() {
    // Arrange
    let mut sim = ClusterSimulation::new(3, 2);

    // Act
    sim.inject_participation(1);

    // Assert
    for peer in 0..3 {
        assert_eq!(sim.cluster_view(peer).pinging_peers(), &[1]);
    }
}

#[test]
fn inject_ready_is_counted_by_every_node() {
    // Arrange
    let mut sim = ClusterSimulation::new(3, 3);

    // Act
    sim.inject_ready(2);

    // Assert
    for peer in 0..3 {
        assert_eq!(sim.cluster_view(peer).collecting_peers(), &[2]);
    }
}
