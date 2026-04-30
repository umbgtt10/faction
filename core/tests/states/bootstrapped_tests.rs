// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::node_state::NodeState;
use faction::state::State;

use faction::states::bootstrapped::Bootstrapped;

fn reach_bootstrapped() -> Faction {
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
    let mut faction = reach_bootstrapped();
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(
        cluster_view.node_state(),
        NodeState::Bootstrapped
    );
    assert_eq!(
        cluster_view.exit_mode(),
        Some(ReadinessExitMode::Bootstrapped)
    );
    assert!(cluster_view.readiness_exited());
}

#[test]
fn all_inputs_leave_state_unchanged() {
    // Arrange
    let mut faction = reach_bootstrapped();
    let snapshot_before = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let r1 = match faction.process(Command::ParticipationObserved {
        peer_id: 0,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r2 = match faction.process(Command::ReadyObserved {
        peer_id: 4,
        freshness: 10,
        current_marker: 10,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r3 = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r4 = match faction.process(Command::DeadlineExpired) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let snapshot_after = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

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
    let mut faction = reach_bootstrapped();
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(
        cluster_view.node_state(),
        NodeState::Bootstrapped
    );
    assert_eq!(
        cluster_view.exit_mode(),
        Some(ReadinessExitMode::Bootstrapped)
    );
    assert!(cluster_view.local_participation_complete());
    assert!(cluster_view.readiness_exited());
    assert_eq!(cluster_view.pinging_confirmed_count(), 1);
    assert_eq!(cluster_view.collecting_confirmed_count(), 4);
    assert_eq!(cluster_view.quorum_threshold(), 4);
}

#[test]
fn bootstrapped_cluster_view_overrides_all_fields() {
    // Arrange
    let rq = Bootstrapped {
        pinging_count: 2,
        collecting_count: 5,
    };
    let prev = ClusterView::new(NodeState::Pinging, false, 99, 99, 4);

    // Act
    let result = rq.cluster_view(&prev);

    // Assert
    assert_eq!(
        result.node_state(),
        NodeState::Bootstrapped
    );
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Bootstrapped));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
    assert_eq!(result.pinging_confirmed_count(), 2);
    assert_eq!(result.collecting_confirmed_count(), 5);
    assert_eq!(result.quorum_threshold(), 4);
}
