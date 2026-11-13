// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::recording_observer::Calls;
use crate::support::recording_observer::RecordingObserver;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::peer_state::PeerState;
use faction_core_validation::cluster_simulation::ClusterSimulation;
use std::cell::RefCell;
use std::rc::Rc;

fn quiet(count: usize) -> Vec<Box<dyn Observer>> {
    (0..count)
        .map(|_| Box::new(NoOpObserver) as Box<dyn Observer>)
        .collect()
}

#[test]
fn all_exited_with_before_any_signal_returns_false() {
    // Arrange
    let mut sim = ClusterSimulation::new(quiet(3), 2);

    // Act
    let all_exited = sim.all_exited_with(Conclusion::Bootstrapped);

    // Assert
    assert!(!all_exited);
}

#[test]
fn cluster_view_reports_the_named_peer() {
    // Arrange
    let mut sim = ClusterSimulation::new(quiet(3), 3);
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
    let mut sim = ClusterSimulation::new(quiet(3), 2);

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
    let mut sim = ClusterSimulation::new(quiet(3), 3);

    // Act
    sim.inject_ready(2);

    // Assert
    for peer in 0..3 {
        assert_eq!(sim.cluster_view(peer).collecting_peers(), &[2]);
    }
}

#[test]
fn new_gives_each_node_its_own_observer() {
    // Arrange
    let calls: Vec<Calls> = (0..3).map(|_| Rc::new(RefCell::new(Vec::new()))).collect();
    let observers = calls
        .iter()
        .map(|calls| Box::new(RecordingObserver::new(Rc::clone(calls))) as Box<dyn Observer>)
        .collect();
    let mut sim = ClusterSimulation::new(observers, 3);

    // Act
    sim.complete_local(1);

    // Assert
    let first: Vec<Command> = calls.iter().map(|calls| calls.borrow()[0].0).collect();
    assert_eq!(
        first,
        vec![
            Command::ReadyObserved { peer_id: 1 },
            Command::LocalParticipationCompleted,
            Command::ReadyObserved { peer_id: 1 },
        ]
    );
}
