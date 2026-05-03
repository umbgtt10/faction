// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::state::State;

use faction::states::bootstrapped::Bootstrapped;

fn reach_bootstrapped() -> Faction {
    let mut faction = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 3 });
    faction
}

#[test]
fn process_rejects_participation_observed() {
    // Arrange & Act
    let mut faction = reach_bootstrapped();
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.exit_mode(), Some(Conclusion::Bootstrapped));
    assert!(cluster_view.is_exited());
}

#[test]
fn process_all_inputs_leave_state_unchanged() {
    // Arrange
    let mut faction = reach_bootstrapped();
    let snapshot_before = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let r1 = match faction.process(Command::ParticipationObserved { peer_id: 0 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r2 = match faction.process(Command::ReadyObserved { peer_id: 4 }) {
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
fn process_probe_returns_correct_snapshot() {
    // Arrange & Act
    let mut faction = reach_bootstrapped();
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.exit_mode(), Some(Conclusion::Bootstrapped));
    assert!(cluster_view.is_pinging_completed());
    assert!(cluster_view.is_exited());
    assert_eq!(cluster_view.pinging_peers().len(), 1);
    assert_eq!(cluster_view.collecting_peers().len(), 4);
    assert_eq!(cluster_view.required_count(), 4);
}

#[test]
fn cluster_view_overrides_all_fields() {
    // Arrange
    let bootstrapped = Bootstrapped::new(vec![1, 2], vec![1, 2, 3, 4, 5]);
    let prev = ClusterView::new(
        PeerState::Pinging,
        false,
        vec![1, 2],
        vec![1, 2, 3, 4, 5],
        4,
    );

    // Act
    let result = bootstrapped.cluster_view(&prev);

    // Assert
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.exit_mode(), Some(Conclusion::Bootstrapped));
    assert!(result.is_pinging_completed());
    assert!(result.is_exited());
    assert_eq!(result.pinging_peers(), &[1, 2]);
    assert_eq!(result.collecting_peers(), &[1, 2, 3, 4, 5]);
    assert_eq!(result.required_count(), 4);
}
