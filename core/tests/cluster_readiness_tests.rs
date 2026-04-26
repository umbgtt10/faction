// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use faction::cluster_readiness::ClusterReadiness;
use faction::cluster_readiness_config::ClusterReadinessConfig;
use faction::cluster_readiness_input::ClusterReadinessInput;
use faction::cluster_readiness_output::ClusterReadinessOutput;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_cluster_readiness_observer::NoOpClusterReadinessObserver;
use faction::output_batch::OutputBatch;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;

fn test_config() -> ClusterReadinessConfig {
    ClusterReadinessConfig::new(
        0,
        vec![0, 1, 2, 3, 4],
        QuorumPolicy::new(4),
        FreshnessPolicy::new(2),
    )
}

fn coordinator() -> ClusterReadiness {
    ClusterReadiness::new(test_config(), Box::new(NoOpClusterReadinessObserver))
}

fn outputs(batch: &OutputBatch) -> Vec<ClusterReadinessOutput> {
    batch.outputs().to_vec()
}

#[test]
fn local_participation_completion_emits_local_ready_broadcast_request() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![
            ClusterReadinessOutput::LocalParticipationCompleted,
            ClusterReadinessOutput::BroadcastLocalReady,
        ]
    );
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(snapshot.local_participation_complete());
    assert!(!snapshot.readiness_exited());
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
    assert_eq!(snapshot.exit_mode(), None);
}

#[test]
fn self_is_counted_in_phase2_on_local_participation_completion() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
    assert_eq!(snapshot.quorum_threshold(), 4);
}

#[test]
fn ready_quorum_reached_exits_by_quorum() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![
            ClusterReadinessOutput::ReadyAccepted { peer_id: 3 },
            ClusterReadinessOutput::ReadyQuorumReached,
            ClusterReadinessOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot.readiness_exited());
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
}

#[test]
fn deadline_expiry_before_quorum_exits_by_deadline() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::DeadlineExpired);
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(snapshot.readiness_exited());
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
}

#[test]
fn duplicate_participation_observation_is_ignored() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::DuplicateParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn duplicate_ready_observation_is_ignored() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::DuplicateReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 2);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn non_member_participation_observation_is_ignored() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 0);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn non_member_ready_observation_is_ignored() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::NonMemberIgnored { peer_id: 99 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 0);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn stale_participation_observation_is_ignored_before_state_mutation() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::StaleParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 0);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn stale_ready_observation_is_ignored_before_state_mutation() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::StaleReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn delayed_participation_within_margin_is_accepted() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn delayed_ready_within_margin_is_accepted() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::DelayedReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 2);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn ready_observation_after_quorum_exit_is_ignored() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 4,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::StaleReadyIgnored { peer_id: 4 }]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
    assert!(snapshot.readiness_exited());
}

#[test]
fn deadline_expiry_after_exit_is_no_op() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let _ = coordinator.apply(ClusterReadinessInput::DeadlineExpired);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::DeadlineExpired);
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert!(outputs.is_empty());
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert!(snapshot.readiness_exited());
}

#[test]
fn readiness_does_not_reopen_after_exit() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let _ = coordinator.apply(ClusterReadinessInput::DeadlineExpired);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::StaleParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert!(snapshot.readiness_exited());
}

#[test]
fn delayed_participation_at_margin_is_accepted() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn delayed_participation_just_inside_margin_is_accepted() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 9,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn delayed_participation_just_outside_margin_is_stale() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::StaleParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 0);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn delayed_ready_at_margin_is_accepted() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::DelayedReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 2);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn delayed_ready_just_inside_margin_is_accepted() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 9,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::DelayedReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 2);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn delayed_ready_just_outside_margin_is_stale() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::StaleReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn local_participation_completion_outputs_match_snapshot_state() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![
            ClusterReadinessOutput::LocalParticipationCompleted,
            ClusterReadinessOutput::BroadcastLocalReady,
        ]
    );
    assert!(snapshot.local_participation_complete());
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
    assert_eq!(snapshot.exit_mode(), None);
}

#[test]
fn quorum_exit_outputs_match_snapshot_state() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![
            ClusterReadinessOutput::ReadyAccepted { peer_id: 3 },
            ClusterReadinessOutput::ReadyQuorumReached,
            ClusterReadinessOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert!(snapshot.readiness_exited());
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
}

#[test]
fn deadline_exit_output_matches_snapshot_state() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::DeadlineExpired);
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert!(snapshot.readiness_exited());
}

#[test]
fn duplicate_participation_output_matches_snapshot_state() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::DuplicateParticipationIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert_eq!(snapshot.phase2_confirmed_count(), 0);
    assert!(!snapshot.readiness_exited());
}

#[test]
fn stale_ready_output_matches_snapshot_state() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 7,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::StaleReadyIgnored { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
    assert_eq!(snapshot.exit_mode(), None);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(!snapshot.readiness_exited());
}

#[test]
fn ready_observed_before_local_participation_completion_is_recorded() {
    // Arrange
    let mut coordinator = coordinator();

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![ClusterReadinessOutput::ReadyAccepted { peer_id: 1 }]
    );
    assert_eq!(snapshot.phase2_confirmed_count(), 1);
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert!(!snapshot.local_participation_complete());
    assert!(!snapshot.readiness_exited());
}

#[test]
fn local_participation_completion_with_preexisting_ready_quorum_exits_immediately() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert_eq!(
        outputs,
        vec![
            ClusterReadinessOutput::LocalParticipationCompleted,
            ClusterReadinessOutput::BroadcastLocalReady,
            ClusterReadinessOutput::ReadyQuorumReached,
            ClusterReadinessOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert!(snapshot.readiness_exited());
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
}

#[test]
fn exit_mode_is_immutable_after_exit() {
    // Arrange
    let mut coordinator = coordinator();

    let _ = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = coordinator.apply(ClusterReadinessInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });

    // Act
    let batch = coordinator.apply(ClusterReadinessInput::DeadlineExpired);
    let outputs = outputs(&batch);
    let snapshot = coordinator.snapshot();

    // Assert
    assert!(outputs.is_empty());
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert!(snapshot.readiness_exited());
}
