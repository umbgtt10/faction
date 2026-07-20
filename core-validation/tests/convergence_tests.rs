// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use faction::conclusion::Conclusion;
use faction::peer_state::PeerState;
use faction_core_validation::cluster_simulation::ClusterSimulation;

#[test]
fn five_nodes_converge_on_quorum() {
    let mut sim = ClusterSimulation::new(5, 4);

    for peer in 1..5 {
        sim.inject_participation(peer);
    }

    for peer in 0..5 {
        sim.complete_local(peer);
    }

    assert!(sim.all_exited_with(Conclusion::Bootstrapped));
}

#[test]
fn not_enough_signals_records_deadline_miss() {
    let mut sim = ClusterSimulation::new(5, 4);

    sim.inject_participation(1);
    sim.complete_local(0);
    sim.inject_ready(1);

    for peer in 0..5 {
        sim.expire_deadline(peer);
    }

    // A missed deadline is recorded, not terminal: every node reports TimedOut
    // yet none has concluded, so they stay receptive.
    for peer in 0..5 {
        let view = sim.cluster_view(peer);
        assert_eq!(view.peer_state(), PeerState::TimedOut);
        assert!(!view.is_concluded());
        assert!(view.deadline_missed());
    }
}

#[test]
fn duplicate_signals_dont_disrupt_convergence() {
    let mut sim = ClusterSimulation::new(5, 4);

    for peer in 1..5 {
        sim.inject_participation(peer);
        sim.inject_participation(peer);
    }

    for peer in 0..5 {
        sim.complete_local(peer);
    }

    assert!(sim.pending_count() == 0);
    assert!(sim.all_exited_with(Conclusion::Bootstrapped));
}
