// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;

#[test]
fn get_snapshot_returns_snapshot_available_with_initial_state() {
    // Arrange
    let config = Config::new(
        0,
        vec![0, 1, 2, 3, 4],
        QuorumPolicy::new(4),
        FreshnessPolicy::new(2),
    );
    let observer = Box::new(NoOpObserver);
    let mut faction = Faction::new(config, observer);

    // Act
    let snapshot = match faction.process(Command::GetSnapshot) {
        ProcessResult::Snapshot { snapshot } => snapshot,
        _ => panic!("expected Snapshot"),
    };

    // Assert
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snapshot.exit_mode(), None);
    assert!(!snapshot.local_participation_complete());
    assert!(!snapshot.readiness_exited());
    assert_eq!(snapshot.phase1_confirmed_count(), 0);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
}

#[test]
fn get_snapshot_does_not_mutate_state() {
    // Arrange
    let config = Config::new(
        0,
        vec![0, 1, 2, 3, 4],
        QuorumPolicy::new(4),
        FreshnessPolicy::new(2),
    );
    let observer = Box::new(NoOpObserver);
    let mut faction = Faction::new(config, observer);

    // Act
    let first = faction.snapshot();
    let _ = faction.process(Command::GetSnapshot);
    let second = faction.snapshot();

    // Assert
    assert_eq!(first, second);
}

#[test]
fn get_snapshot_works_after_valid_inputs() {
    // Arrange
    let config = Config::new(
        0,
        vec![0, 1, 2, 3, 4],
        QuorumPolicy::new(4),
        FreshnessPolicy::new(2),
    );
    let observer = Box::new(NoOpObserver);
    let mut faction = Faction::new(config, observer);
    let _ = faction.process(Command::ParticipationObserved {
        peer_id: 0,
        freshness: 5,
        current_marker: 5,
    });

    // Act
    let snapshot = match faction.process(Command::GetSnapshot) {
        ProcessResult::Snapshot { snapshot } => snapshot,
        _ => panic!("expected Snapshot"),
    };

    // Assert
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
}
