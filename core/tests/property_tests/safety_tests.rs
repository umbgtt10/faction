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
use proptest::prelude::*;

fn test_config() -> Config {
    Config::new(
        0,
        vec![0, 1, 2, 3, 4],
        QuorumPolicy::new(4),
        FreshnessPolicy::new(2),
    )
}

fn coordinator() -> Faction {
    Faction::new(test_config(), Box::new(NoOpObserver))
}

fn input_strategy() -> impl Strategy<Value = Command> {
    let participation =
        (0u64..=6, 0u64..=12, 0u64..=12).prop_map(|(peer_id, freshness, current_marker)| {
            Command::ParticipationObserved {
                peer_id,
                freshness,
                current_marker,
            }
        });
    let ready =
        (0u64..=6, 0u64..=12, 0u64..=12).prop_map(|(peer_id, freshness, current_marker)| {
            Command::ReadyObserved {
                peer_id,
                freshness,
                current_marker,
            }
        });

    prop_oneof![
        participation,
        ready,
        Just(Command::LocalParticipationCompleted),
        Just(Command::DeadlineExpired),
    ]
}

fn assert_post_exit_inputs_do_not_change_any_field(
    previous: ClusterView,
    current: ClusterView,
) -> Result<(), TestCaseError> {
    if previous.is_exited() {
        prop_assert_eq!(current, previous);
    }
    Ok(())
}

proptest! {
    #[test]
    fn counts_never_exceed_peer_count(commands in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for command in commands {
            let _ = coordinator.process(command);
            let cluster_view = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            prop_assert!(cluster_view.pinging_peers().len() <= 5);
            prop_assert!(cluster_view.collecting_peers().len() <= 5);
        }
    }

    #[test]
    fn required_count_never_changes(commands in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for command in commands {
            let _ = coordinator.process(command);
            let cluster_view = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            prop_assert_eq!(cluster_view.required_count(), 4);
        }
    }

    #[test]
    fn local_participation_completion_is_idempotent(
        commands in prop::collection::vec(input_strategy(), 0..128)
    ) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for command in commands {
            let _ = coordinator.process(command);
        }

        let previous = match coordinator.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        };
        let first_status = coordinator.process(Command::LocalParticipationCompleted);
        let after_first = match coordinator.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        };
        let second_status = coordinator.process(Command::LocalParticipationCompleted);
        let after_second = match coordinator.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        };

        // Assert
        if previous.is_pinging_completed() || previous.is_exited() {
            let first_rejected = matches!(first_status, ProcessResult::Rejected { .. });
            prop_assert!(first_rejected);
            prop_assert_eq!(after_first.clone(), previous);
        }
        let second_rejected = matches!(second_status, ProcessResult::Rejected { .. });
        prop_assert!(second_rejected);
        prop_assert_eq!(after_second, after_first);
    }

    #[test]
    fn post_exit_inputs_never_change_any_field(
        commands in prop::collection::vec(input_strategy(), 0..128)
    ) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for command in commands {
            let previous = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            let _ = coordinator.process(command);
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            assert_post_exit_inputs_do_not_change_any_field(previous, current)?;
        }
    }
}
