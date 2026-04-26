// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_readiness::ClusterReadiness;
use faction::cluster_readiness_config::ClusterReadinessConfig;
use faction::cluster_readiness_input::ClusterReadinessInput;
use faction::cluster_readiness_snapshot::ClusterReadinessSnapshot;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_cluster_readiness_observer::NoOpClusterReadinessObserver;
use faction::quorum_policy::QuorumPolicy;
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

fn assert_post_exit_inputs_do_not_change_any_field(
    previous: ClusterReadinessSnapshot,
    current: ClusterReadinessSnapshot,
) -> Result<(), TestCaseError> {
    if previous.readiness_exited() {
        prop_assert_eq!(current, previous);
    }
    Ok(())
}

proptest! {
    #[test]
    fn counts_never_exceed_peer_count(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let _ = coordinator.apply(input);
            let snapshot = coordinator.snapshot();

            // Assert
            prop_assert!(snapshot.phase1_confirmed_count() <= 5);
            prop_assert!(snapshot.phase2_confirmed_count() <= 5);
        }
    }

    #[test]
    fn quorum_threshold_never_changes(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let _ = coordinator.apply(input);
            let snapshot = coordinator.snapshot();

            // Assert
            prop_assert_eq!(snapshot.quorum_threshold(), 4);
        }
    }

    #[test]
    fn local_participation_completion_is_idempotent(
        inputs in prop::collection::vec(input_strategy(), 0..128)
    ) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let _ = coordinator.apply(input);
        }

        let previous = coordinator.snapshot();
        let first_outputs = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
        let after_first = coordinator.snapshot();
        let second_outputs = coordinator.apply(ClusterReadinessInput::LocalParticipationCompleted);
        let after_second = coordinator.snapshot();

        // Assert
        if previous.local_participation_complete() || previous.readiness_exited() {
            prop_assert!(first_outputs.is_empty());
            prop_assert_eq!(after_first, previous);
        }
        prop_assert!(second_outputs.is_empty());
        prop_assert_eq!(after_second, after_first);
    }

    #[test]
    fn post_exit_inputs_never_change_any_field(
        inputs in prop::collection::vec(input_strategy(), 0..128)
    ) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let previous = coordinator.snapshot();
            let _ = coordinator.apply(input);
            let current = coordinator.snapshot();

            // Assert
            assert_post_exit_inputs_do_not_change_any_field(previous, current)?;
        }
    }
}
