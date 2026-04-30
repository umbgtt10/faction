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
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::snapshot::Snapshot;
use faction::state_snapshot::StateSnapshot;
use faction::states::ready_by_quorum::ReadyByQuorum;

fn reach_ready_by_quorum() -> Faction {
    let mut faction = Faction::new(
        Config::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(4),
            FreshnessPolicy::new(2),
        ),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = faction.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = faction.process(Command::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });
    faction
}

#[test]
fn deal_rejects_participation_observed() {
    // Arrange & Act
    let faction = reach_ready_by_quorum();
    let snapshot = faction.snapshot();

    // Assert
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot.readiness_exited());
}

#[test]
fn all_inputs_leave_state_unchanged() {
    // Arrange
    let mut faction = reach_ready_by_quorum();
    let snapshot_before = faction.snapshot();

    // Act
    let r1 = match faction.process(Command::ParticipationObserved {
        peer_id: 0,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Snapshot { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r2 = match faction.process(Command::ReadyObserved {
        peer_id: 4,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Snapshot { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r3 = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Snapshot { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r4 = match faction.process(Command::DeadlineExpired) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Snapshot { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let snapshot_after = faction.snapshot();

    // Assert
    assert!(r1.is_empty());
    assert!(r2.is_empty());
    assert!(r3.is_empty());
    assert!(r4.is_empty());
    assert_eq!(snapshot_before, snapshot_after);
}

#[test]
fn vibe_check_returns_correct_snapshot() {
    // Arrange & Act
    let faction = reach_ready_by_quorum();
    let snapshot = faction.snapshot();

    // Assert
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot.local_participation_complete());
    assert!(snapshot.readiness_exited());
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
    assert_eq!(snapshot.quorum_threshold(), 4);
}

#[test]
fn ready_by_quorum_state_snapshot_overrides_all_fields() {
    // Arrange
    let rq = ReadyByQuorum {
        phase1_count: 2,
        phase2_count: 5,
    };
    let prev = Snapshot::new(
        ReadinessLifecycleState::Phase1Active,
        None,
        false,
        false,
        99,
        99,
        4,
    );

    // Act
    let result = rq.state_snapshot(&prev);

    // Assert
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.phase1_confirmed_count(), 2);
    assert_eq!(result.phase2_confirmed_count(), 5);
    assert_eq!(result.quorum_threshold(), 4);
}
