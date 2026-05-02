// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use faction::conclusion::Conclusion;
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
fn not_enough_signals_triggers_deadline() {
    let mut sim = ClusterSimulation::new(5, 4);

    sim.inject_participation(1);
    sim.complete_local(0);
    sim.inject_ready(1);

    for peer in 0..5 {
        sim.expire_deadline(peer);
    }

    assert!(sim.all_exited_with(Conclusion::TimedOut));
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
