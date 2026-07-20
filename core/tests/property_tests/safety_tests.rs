// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use proptest::prelude::*;

fn test_config() -> Config {
    Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4))
}

fn faction() -> Faction {
    Faction::new(test_config(), Box::new(NoOpObserver))
}

fn input_strategy() -> impl Strategy<Value = Command> {
    let participation = (0u64..=6).prop_map(|peer_id| Command::ParticipationObserved { peer_id });
    let ready = (0u64..=6).prop_map(|peer_id| Command::ReadyObserved { peer_id });

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
    if previous.is_concluded() {
        prop_assert_eq!(current, previous);
    }
    Ok(())
}

proptest! {
    #[test]
    fn counts_never_exceed_peer_count(commands in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();

        // Act
        for command in commands {
            let _ = faction.process(command);
            let cluster_view = match faction.process(Command::Probe) {
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
        let mut faction = faction();

        // Act
        for command in commands {
            let _ = faction.process(command);
            let cluster_view = match faction.process(Command::Probe) {
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
        let mut faction = faction();

        // Act
        for command in commands {
            let _ = faction.process(command);
        }

        let previous = match faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        };
        let first_status = faction.process(Command::LocalParticipationCompleted);
        let after_first = match faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        };
        let second_status = faction.process(Command::LocalParticipationCompleted);
        let after_second = match faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        };

        // Assert
        if previous.is_pinging_completed() || previous.is_concluded() {
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
        let mut faction = faction();

        // Act
        for command in commands {
            let previous = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            let _ = faction.process(command);
            let current = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            assert_post_exit_inputs_do_not_change_any_field(previous, current)?;
        }
    }
}
