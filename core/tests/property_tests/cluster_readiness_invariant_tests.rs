// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_readiness::ClusterReadiness;
use faction::cluster_readiness_config::ClusterReadinessConfig;
use faction::cluster_readiness_input::ClusterReadinessInput;
use faction::cluster_readiness_output::ClusterReadinessOutput;
use faction::cluster_readiness_snapshot::ClusterReadinessSnapshot;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_cluster_readiness_observer::NoOpClusterReadinessObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use proptest::prelude::*;

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

fn input_strategy() -> impl Strategy<Value = ClusterReadinessInput> {
    let participation =
        (0u64..=6, 0u64..=12, 0u64..=12).prop_map(|(peer_id, freshness, current_marker)| {
            ClusterReadinessInput::ParticipationObserved {
                peer_id,
                freshness,
                current_marker,
            }
        });
    let ready =
        (0u64..=6, 0u64..=12, 0u64..=12).prop_map(|(peer_id, freshness, current_marker)| {
            ClusterReadinessInput::ReadyObserved {
                peer_id,
                freshness,
                current_marker,
            }
        });

    prop_oneof![
        participation,
        ready,
        Just(ClusterReadinessInput::LocalParticipationCompleted),
        Just(ClusterReadinessInput::DeadlineExpired),
    ]
}

fn outputs_contain_stale(outputs: &[ClusterReadinessOutput]) -> bool {
    outputs.iter().any(|output| {
        matches!(
            output,
            ClusterReadinessOutput::StaleParticipationIgnored { .. }
                | ClusterReadinessOutput::StaleReadyIgnored { .. }
        )
    })
}

fn outputs_contain_non_member(outputs: &[ClusterReadinessOutput]) -> bool {
    outputs
        .iter()
        .any(|output| matches!(output, ClusterReadinessOutput::NonMemberIgnored { .. }))
}

fn outputs_contain_duplicate(outputs: &[ClusterReadinessOutput]) -> bool {
    outputs.iter().any(|output| {
        matches!(
            output,
            ClusterReadinessOutput::DuplicateParticipationIgnored { .. }
                | ClusterReadinessOutput::DuplicateReadyIgnored { .. }
        )
    })
}

fn assert_counts_do_not_decrease(
    previous: ClusterReadinessSnapshot,
    current: ClusterReadinessSnapshot,
) -> Result<(), TestCaseError> {
    prop_assert!(current.phase1_confirmed_count() >= previous.phase1_confirmed_count());
    prop_assert!(current.phase2_confirmed_count() >= previous.phase2_confirmed_count());
    Ok(())
}

fn assert_stale_outputs_do_not_mutate_state(
    previous: ClusterReadinessSnapshot,
    current: ClusterReadinessSnapshot,
    outputs: &[ClusterReadinessOutput],
) -> Result<(), TestCaseError> {
    if outputs_contain_stale(outputs) {
        prop_assert_eq!(
            current.phase1_confirmed_count(),
            previous.phase1_confirmed_count()
        );
        prop_assert_eq!(
            current.phase2_confirmed_count(),
            previous.phase2_confirmed_count()
        );
        prop_assert_eq!(current.lifecycle_state(), previous.lifecycle_state());
        prop_assert_eq!(current.exit_mode(), previous.exit_mode());
        prop_assert_eq!(
            current.local_participation_complete(),
            previous.local_participation_complete()
        );
        prop_assert_eq!(current.readiness_exited(), previous.readiness_exited());
    }
    Ok(())
}

fn assert_non_member_outputs_do_not_mutate_state(
    previous: ClusterReadinessSnapshot,
    current: ClusterReadinessSnapshot,
    outputs: &[ClusterReadinessOutput],
) -> Result<(), TestCaseError> {
    if outputs_contain_non_member(outputs) {
        prop_assert_eq!(
            current.phase1_confirmed_count(),
            previous.phase1_confirmed_count()
        );
        prop_assert_eq!(
            current.phase2_confirmed_count(),
            previous.phase2_confirmed_count()
        );
        prop_assert_eq!(current.lifecycle_state(), previous.lifecycle_state());
        prop_assert_eq!(current.exit_mode(), previous.exit_mode());
        prop_assert_eq!(
            current.local_participation_complete(),
            previous.local_participation_complete()
        );
        prop_assert_eq!(current.readiness_exited(), previous.readiness_exited());
    }
    Ok(())
}

fn assert_duplicate_outputs_do_not_mutate_counts(
    previous: ClusterReadinessSnapshot,
    current: ClusterReadinessSnapshot,
    outputs: &[ClusterReadinessOutput],
) -> Result<(), TestCaseError> {
    if outputs_contain_duplicate(outputs) {
        prop_assert_eq!(
            current.phase1_confirmed_count(),
            previous.phase1_confirmed_count()
        );
        prop_assert_eq!(
            current.phase2_confirmed_count(),
            previous.phase2_confirmed_count()
        );
        prop_assert_eq!(current.lifecycle_state(), previous.lifecycle_state());
        prop_assert_eq!(current.exit_mode(), previous.exit_mode());
        prop_assert_eq!(
            current.local_participation_complete(),
            previous.local_participation_complete()
        );
        prop_assert_eq!(current.readiness_exited(), previous.readiness_exited());
    }
    Ok(())
}

proptest! {
    #[test]
    fn exit_mode_never_changes_after_exit(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut exited_mode = None;

        // Act
        for input in inputs {
            let _ = coordinator.apply(input);
            let snapshot = coordinator.snapshot();

            // Assert
            if let Some(mode) = exited_mode {
                prop_assert_eq!(snapshot.exit_mode(), Some(mode));
            } else if let Some(mode) = snapshot.exit_mode() {
                exited_mode = Some(mode);
            }
        }
    }

    #[test]
    fn once_exited_state_never_returns_to_active(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut has_exited = false;

        // Act
        for input in inputs {
            let _ = coordinator.apply(input);
            let snapshot = coordinator.snapshot();

            // Assert
            if snapshot.readiness_exited() {
                has_exited = true;
                prop_assert!(matches!(
                    snapshot.lifecycle_state(),
                    ReadinessLifecycleState::ReadyByQuorum | ReadinessLifecycleState::ReadyByDeadline
                ));
            }

            if has_exited {
                prop_assert!(snapshot.readiness_exited());
                prop_assert!(matches!(
                    snapshot.lifecycle_state(),
                    ReadinessLifecycleState::ReadyByQuorum | ReadinessLifecycleState::ReadyByDeadline
                ));
            }
        }
    }

    #[test]
    fn phase1_count_never_decreases(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut previous = coordinator.snapshot();

        // Act
        for input in inputs {
            let _ = coordinator.apply(input);
            let current = coordinator.snapshot();

            // Assert
            prop_assert!(current.phase1_confirmed_count() >= previous.phase1_confirmed_count());
            previous = current;
        }
    }

    #[test]
    fn phase2_count_never_decreases(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut previous = coordinator.snapshot();

        // Act
        for input in inputs {
            let _ = coordinator.apply(input);
            let current = coordinator.snapshot();

            // Assert
            prop_assert!(current.phase2_confirmed_count() >= previous.phase2_confirmed_count());
            previous = current;
        }
    }

    #[test]
    fn stale_inputs_never_mutate_counts(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let previous = coordinator.snapshot();
            let batch = coordinator.apply(input);
            let current = coordinator.snapshot();

            // Assert
            assert_counts_do_not_decrease(previous, current)?;
            assert_stale_outputs_do_not_mutate_state(previous, current, batch.outputs())?;
        }
    }

    #[test]
    fn non_member_inputs_never_mutate_counts(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let previous = coordinator.snapshot();
            let batch = coordinator.apply(input);
            let current = coordinator.snapshot();

            // Assert
            assert_counts_do_not_decrease(previous, current)?;
            assert_non_member_outputs_do_not_mutate_state(previous, current, batch.outputs())?;
        }
    }

    #[test]
    fn readiness_exits_at_most_once(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut has_exited = false;

        // Act
        for input in inputs {
            let previous = coordinator.snapshot();
            let _ = coordinator.apply(input);
            let current = coordinator.snapshot();

            // Assert
            if has_exited {
                prop_assert!(current.readiness_exited());
            }
            if previous.readiness_exited() {
                prop_assert!(current.readiness_exited());
            }
            if current.readiness_exited() {
                has_exited = true;
            }
        }
    }

    #[test]
    fn duplicate_inputs_never_increase_counts(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let previous = coordinator.snapshot();
            let batch = coordinator.apply(input);
            let current = coordinator.snapshot();

            // Assert
            assert_counts_do_not_decrease(previous, current)?;
            assert_duplicate_outputs_do_not_mutate_counts(previous, current, batch.outputs())?;
        }
    }

    #[test]
    fn quorum_exit_implies_local_participation_completion(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let _ = coordinator.apply(input);
            let snapshot = coordinator.snapshot();

            // Assert
            if snapshot.exit_mode() == Some(ReadinessExitMode::Quorum) {
                prop_assert!(snapshot.local_participation_complete());
                prop_assert!(snapshot.readiness_exited());
                prop_assert_eq!(
                    snapshot.lifecycle_state(),
                    ReadinessLifecycleState::ReadyByQuorum
                );
            }
        }
    }

    #[test]
    fn deadline_exit_implies_exited_state(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let _ = coordinator.apply(input);
            let snapshot = coordinator.snapshot();

            // Assert
            if snapshot.exit_mode() == Some(ReadinessExitMode::Deadline) {
                prop_assert!(snapshot.readiness_exited());
                prop_assert_eq!(
                    snapshot.lifecycle_state(),
                    ReadinessLifecycleState::ReadyByDeadline
                );
            }
        }
    }
}
