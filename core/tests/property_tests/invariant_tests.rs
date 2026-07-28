// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use proptest::prelude::*;

fn test_config() -> Config {
    Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4))
}

fn faction() -> Faction {
    Faction::new(test_config(), Box::new(NoOpObserver))
}

fn command_strategy() -> impl Strategy<Value = Command> {
    let participation = (0u64..=6).prop_map(|peer_id| Command::ParticipationObserved { peer_id });
    let ready = (0u64..=6).prop_map(|peer_id| Command::ReadyObserved { peer_id });
    let join_requested = (0u64..=6).prop_map(|peer_id| Command::JoinRequested { peer_id });
    let join_approved = (0u64..=6).prop_map(|peer_id| Command::JoinApproved { peer_id });
    let join_rejected = (0u64..=6).prop_map(|peer_id| Command::JoinRejected { peer_id });

    prop_oneof![
        participation,
        ready,
        Just(Command::LocalParticipationCompleted),
        Just(Command::DeadlineExpired),
        join_requested,
        join_approved,
        join_rejected,
    ]
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
        prop_assert_eq!(current.conclusion(), previous.conclusion());
        prop_assert_eq!(
            current.is_pinging_completed(),
            previous.is_pinging_completed()
        );
        prop_assert_eq!(current.is_concluded(), previous.is_concluded());
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
        prop_assert_eq!(current.conclusion(), previous.conclusion());
        prop_assert_eq!(
            current.is_pinging_completed(),
            previous.is_pinging_completed()
        );
        prop_assert_eq!(current.is_concluded(), previous.is_concluded());
    }
    Ok(())
}

proptest! {
    #[test]
    fn exit_mode_never_changes_after_exit(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();
        let mut exited_mode = None;

        // Act
        for command in commands {
            let _ = faction.process(command);
            let cluster_view = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            if let Some(mode) = exited_mode {
                prop_assert_eq!(cluster_view.conclusion(), Some(mode));
            } else if let Some(mode) = cluster_view.conclusion() {
                exited_mode = Some(mode);
            }
        }
    }

    #[test]
    fn once_exited_state_never_returns_to_active(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();
        let mut has_exited = false;

        // Act
        for command in commands {
            let _ = faction.process(command);
            let cluster_view = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            if cluster_view.is_concluded() {
                has_exited = true;
                prop_assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
            }

            if has_exited {
                prop_assert!(cluster_view.is_concluded());
                prop_assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
            }
        }
    }

    #[test]
    fn pinged_peers_count_never_decreases(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();
        let mut previous = match faction.process(Command::Probe) {
    ProcessResult::Probed { cluster_view, .. } => cluster_view,
    _ => unreachable!(),
};

        // Act
        for command in commands {
            let _ = faction.process(command);
            let current = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            prop_assert!(current.pinging_peers().len() >= previous.pinging_peers().len());
            previous = current;
        }
    }

    #[test]
    fn collected_peers_count_never_decreases(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();
        let mut previous = match faction.process(Command::Probe) {
    ProcessResult::Probed { cluster_view, .. } => cluster_view,
    _ => unreachable!(),
};

        // Act
        for command in commands {
            let _ = faction.process(command);
            let current = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            prop_assert!(current.collecting_peers().len() >= previous.collecting_peers().len());
            previous = current;
        }
    }

    #[test]
    fn non_member_commands_never_mutate_counts(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();

        // Act
        for command in commands {
            let previous = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            let batch = match faction.process(command) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let current = match faction.process(Command::Probe) {
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
        let mut faction = faction();
        let mut has_exited = false;

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
            if has_exited {
                prop_assert!(current.is_concluded());
            }
            if previous.is_concluded() {
                prop_assert!(current.is_concluded());
            }
            if current.is_concluded() {
                has_exited = true;
            }
        }
    }

    #[test]
    fn duplicate_commands_never_increase_counts(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();

        // Act
        for command in commands {
            let previous = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            let batch = match faction.process(command) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let current = match faction.process(Command::Probe) {
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
        let mut faction = faction();

        // Act
        for command in commands {
            let _ = faction.process(command);
            let cluster_view = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            if cluster_view.conclusion() == Some(Conclusion::Bootstrapped) {
                prop_assert!(cluster_view.is_pinging_completed());
                prop_assert!(cluster_view.is_concluded());
                prop_assert_eq!(
                    cluster_view.peer_state(),
                    PeerState::Bootstrapped
                );
            }
        }
    }

    #[test]
    fn deadline_missed_is_reported_but_never_concludes(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();

        // Act
        for command in commands {
            let _ = faction.process(command);
            let cluster_view = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert — a missed deadline surfaces as TimedOut but stays receptive
            if cluster_view.deadline_missed()
                && cluster_view.peer_state() == PeerState::TimedOut
            {
                prop_assert!(!cluster_view.is_concluded());
                prop_assert_eq!(cluster_view.conclusion(), None);
            }
        }
    }
}
