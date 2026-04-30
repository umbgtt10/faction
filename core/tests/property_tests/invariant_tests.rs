// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::exit_mode::ExitMode;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
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

fn command_strategy() -> impl Strategy<Value = Command> {
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

fn outputs_contain_stale(outputs: &[Outcome]) -> bool {
    outputs.iter().any(|output| {
        matches!(
            output,
            Outcome::StaleParticipationIgnored { .. } | Outcome::StaleReadyIgnored { .. }
        )
    })
}

fn outputs_contain_non_member(outputs: &[Outcome]) -> bool {
    outputs
        .iter()
        .any(|output| matches!(output, Outcome::NonMemberIgnored { .. }))
}

fn outputs_contain_duplicate(outputs: &[Outcome]) -> bool {
    outputs.iter().any(|output| {
        matches!(
            output,
            Outcome::DuplicateParticipationIgnored { .. } | Outcome::DuplicateReadyIgnored { .. }
        )
    })
}

fn assert_counts_do_not_decrease(
    previous: ClusterView,
    current: ClusterView,
) -> Result<(), TestCaseError> {
    prop_assert!(current.pinging_peers().len() >= previous.pinging_peers().len());
    prop_assert!(current.collecting_peers().len() >= previous.collecting_peers().len());
    Ok(())
}

fn assert_stale_outputs_do_not_mutate_state(
    previous: ClusterView,
    current: ClusterView,
    outputs: &[Outcome],
) -> Result<(), TestCaseError> {
    if outputs_contain_stale(outputs) {
        prop_assert_eq!(
            current.pinging_peers().len(),
            previous.pinging_peers().len()
        );
        prop_assert_eq!(
            current.collecting_peers().len(),
            previous.collecting_peers().len()
        );
        prop_assert_eq!(current.exit_mode(), previous.exit_mode());
        prop_assert_eq!(
            current.is_pinging_completed(),
            previous.is_pinging_completed()
        );
        prop_assert_eq!(current.is_exited(), previous.is_exited());
    }
    Ok(())
}

fn assert_non_member_outputs_do_not_mutate_state(
    previous: ClusterView,
    current: ClusterView,
    outputs: &[Outcome],
) -> Result<(), TestCaseError> {
    if outputs_contain_non_member(outputs) {
        prop_assert_eq!(
            current.pinging_peers().len(),
            previous.pinging_peers().len()
        );
        prop_assert_eq!(
            current.collecting_peers().len(),
            previous.collecting_peers().len()
        );
        prop_assert_eq!(current.exit_mode(), previous.exit_mode());
        prop_assert_eq!(
            current.is_pinging_completed(),
            previous.is_pinging_completed()
        );
        prop_assert_eq!(current.is_exited(), previous.is_exited());
    }
    Ok(())
}

fn assert_duplicate_outputs_do_not_mutate_counts(
    previous: ClusterView,
    current: ClusterView,
    outputs: &[Outcome],
) -> Result<(), TestCaseError> {
    if outputs_contain_duplicate(outputs) {
        prop_assert_eq!(
            current.pinging_peers().len(),
            previous.pinging_peers().len()
        );
        prop_assert_eq!(
            current.collecting_peers().len(),
            previous.collecting_peers().len()
        );
        prop_assert_eq!(current.exit_mode(), previous.exit_mode());
        prop_assert_eq!(
            current.is_pinging_completed(),
            previous.is_pinging_completed()
        );
        prop_assert_eq!(current.is_exited(), previous.is_exited());
    }
    Ok(())
}

proptest! {
    #[test]
    fn exit_mode_never_changes_after_exit(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut exited_mode = None;

        // Act
        for command in commands {
            let _ = coordinator.process(command);
            let cluster_view = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            if let Some(mode) = exited_mode {
                prop_assert_eq!(cluster_view.exit_mode(), Some(mode));
            } else if let Some(mode) = cluster_view.exit_mode() {
                exited_mode = Some(mode);
            }
        }
    }

    #[test]
    fn once_exited_state_never_returns_to_active(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut has_exited = false;

        // Act
        for command in commands {
            let _ = coordinator.process(command);
            let cluster_view = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            if cluster_view.is_exited() {
                has_exited = true;
                prop_assert!(matches!(
                    cluster_view.peer_state(),
                    PeerState::Bootstrapped | PeerState::TimedOut
                ));
            }

            if has_exited {
                prop_assert!(cluster_view.is_exited());
                prop_assert!(matches!(
                    cluster_view.peer_state(),
                    PeerState::Bootstrapped | PeerState::TimedOut
                ));
            }
        }
    }

    #[test]
    fn pinging_count_never_decreases(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut previous = match coordinator.process(Command::Probe) {
    ProcessResult::Probed { cluster_view, .. } => cluster_view,
    _ => unreachable!(),
};

        // Act
        for command in commands {
            let _ = coordinator.process(command);
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            prop_assert!(current.pinging_peers().len() >= previous.pinging_peers().len());
            previous = current;
        }
    }

    #[test]
    fn collecting_count_never_decreases(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut previous = match coordinator.process(Command::Probe) {
    ProcessResult::Probed { cluster_view, .. } => cluster_view,
    _ => unreachable!(),
};

        // Act
        for command in commands {
            let _ = coordinator.process(command);
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            prop_assert!(current.collecting_peers().len() >= previous.collecting_peers().len());
            previous = current;
        }
    }

    #[test]
    fn stale_commands_never_mutate_counts(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for command in commands {
            let previous = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            let batch = match coordinator.process(command) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            assert_counts_do_not_decrease(previous.clone(), current.clone())?;
            assert_stale_outputs_do_not_mutate_state(previous, current, &batch)?;
        }
    }

    #[test]
    fn non_member_commands_never_mutate_counts(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for command in commands {
            let previous = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            let batch = match coordinator.process(command) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            assert_counts_do_not_decrease(previous.clone(), current.clone())?;
            assert_non_member_outputs_do_not_mutate_state(previous, current, &batch)?;
        }
    }

    #[test]
    fn exits_at_most_once(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut has_exited = false;

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
            if has_exited {
                prop_assert!(current.is_exited());
            }
            if previous.is_exited() {
                prop_assert!(current.is_exited());
            }
            if current.is_exited() {
                has_exited = true;
            }
        }
    }

    #[test]
    fn duplicate_commands_never_increase_counts(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for command in commands {
            let previous = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            let batch = match coordinator.process(command) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            assert_counts_do_not_decrease(previous.clone(), current.clone())?;
            assert_duplicate_outputs_do_not_mutate_counts(previous, current, &batch)?;
        }
    }

    #[test]
    fn quorum_exit_implies_completed_pinging(commands in prop::collection::vec(command_strategy(), 0..128)) {
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
            if cluster_view.exit_mode() == Some(ExitMode::Bootstrapped) {
                prop_assert!(cluster_view.is_pinging_completed());
                prop_assert!(cluster_view.is_exited());
                prop_assert_eq!(
                    cluster_view.peer_state(),
                    PeerState::Bootstrapped
                );
            }
        }
    }

    #[test]
    fn deadline_exit_implies_exited_state(commands in prop::collection::vec(command_strategy(), 0..128)) {
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
            if cluster_view.exit_mode() == Some(ExitMode::TimedOut) {
                prop_assert!(cluster_view.is_exited());
                prop_assert_eq!(
                    cluster_view.peer_state(),
                    PeerState::TimedOut
                );
            }
        }
    }
}
