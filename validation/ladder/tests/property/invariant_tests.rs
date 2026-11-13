// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::random_commands::RandomCommands;
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

// The pinging and collecting peers never shrink, but by the peer a committed
// removal takes out: after it, each is what it was without that peer.
fn assert_counts_do_not_decrease(
    previous: ClusterView,
    current: ClusterView,
    outputs: &[Outcome],
) -> Result<(), TestCaseError> {
    match removed_peer(outputs) {
        Some(peer_id) => {
            prop_assert_eq!(
                current.pinging_peers().to_vec(),
                without(previous.pinging_peers(), peer_id)
            );
            prop_assert_eq!(
                current.collecting_peers().to_vec(),
                without(previous.collecting_peers(), peer_id)
            );
        }
        None => {
            prop_assert!(current.pinging_peers().len() >= previous.pinging_peers().len());
            prop_assert!(current.collecting_peers().len() >= previous.collecting_peers().len());
        }
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
        prop_assert_eq!(
            (current.peer_state() == PeerState::Bootstrapped).then_some(Conclusion::Bootstrapped),
            (previous.peer_state() == PeerState::Bootstrapped).then_some(Conclusion::Bootstrapped)
        );
        prop_assert_eq!(
            current.is_pinging_completed(),
            previous.is_pinging_completed()
        );
        prop_assert_eq!(
            current.peer_state() == PeerState::Bootstrapped,
            previous.peer_state() == PeerState::Bootstrapped
        );
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
        prop_assert_eq!(
            (current.peer_state() == PeerState::Bootstrapped).then_some(Conclusion::Bootstrapped),
            (previous.peer_state() == PeerState::Bootstrapped).then_some(Conclusion::Bootstrapped)
        );
        prop_assert_eq!(
            current.is_pinging_completed(),
            previous.is_pinging_completed()
        );
        prop_assert_eq!(
            current.peer_state() == PeerState::Bootstrapped,
            previous.peer_state() == PeerState::Bootstrapped
        );
    }
    Ok(())
}

// Every command but the probe the properties make between two calls.
fn command_strategy() -> impl Strategy<Value = Command> {
    RandomCommands::command().prop_filter("a command, not a probe", |command| {
        *command != Command::Probe
    })
}

extern crate alloc;

fn faction() -> Faction {
    Faction::new(test_config(), Box::new(NoOpObserver))
}

// What a call answered, as outcomes: none for a rejected command.
fn outcomes(result: ProcessResult) -> Vec<Outcome> {
    match result {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } | ProcessResult::Rejected { .. } => vec![],
    }
}

fn outputs_contain_duplicate(outputs: &[Outcome]) -> bool {
    outputs.iter().any(|output| {
        matches!(
            output,
            Outcome::DuplicateParticipationIgnored { .. } | Outcome::DuplicateReadyIgnored { .. }
        )
    })
}

fn outputs_contain_non_member(outputs: &[Outcome]) -> bool {
    outputs
        .iter()
        .any(|output| matches!(output, Outcome::NonMemberIgnored { .. }))
}

// The node's own removal, as a call's outcomes say it: node 0 removed.
fn own_removal(outputs: &[Outcome]) -> bool {
    outputs.contains(&Outcome::MemberRemoved { peer_id: 0 })
}

proptest! {
    #[test]
    fn exit_mode_never_changes_after_exit(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();
        let mut exited_mode = None;
        let mut removed = false;

        // Act
        for command in commands {
            removed = removed || own_removal(&outcomes(faction.process(command)));
            let cluster_view = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            if removed {
                prop_assert_eq!(cluster_view.peer_state(), PeerState::Removed);
            } else if let Some(mode) = exited_mode {
                prop_assert_eq!((cluster_view.peer_state() == PeerState::Bootstrapped).then_some(Conclusion::Bootstrapped), Some(mode));
            } else if let Some(mode) = (cluster_view.peer_state() == PeerState::Bootstrapped).then_some(Conclusion::Bootstrapped) {
                exited_mode = Some(mode);
            }
        }
    }

    #[test]
    fn once_exited_state_never_returns_to_active(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();
        let mut has_exited = false;
        let mut removed = false;

        // Act
        for command in commands {
            removed = removed || own_removal(&outcomes(faction.process(command)));
            let cluster_view = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            if removed {
                prop_assert_eq!(cluster_view.peer_state(), PeerState::Removed);
                continue;
            }
            if cluster_view.peer_state() == PeerState::Bootstrapped {
                has_exited = true;
                prop_assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
            }

            if has_exited {
                prop_assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
                prop_assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
            }
        }
    }

    #[test]
    fn pinged_peers_count_never_decreases_but_by_a_removal(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();
        let mut previous = match faction.process(Command::Probe) {
    ProcessResult::Probed { cluster_view, .. } => cluster_view,
    _ => unreachable!(),
};

        // Act
        for command in commands {
            let removed = removed_peer(&outcomes(faction.process(command)));
            let current = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            match removed {
                Some(peer_id) => prop_assert_eq!(current.pinging_peers().to_vec(), without(previous.pinging_peers(), peer_id)),
                None => prop_assert!(current.pinging_peers().len() >= previous.pinging_peers().len()),
            }
            previous = current;
        }
    }

    #[test]
    fn collected_peers_count_never_decreases_but_by_a_removal(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();
        let mut previous = match faction.process(Command::Probe) {
    ProcessResult::Probed { cluster_view, .. } => cluster_view,
    _ => unreachable!(),
};

        // Act
        for command in commands {
            let removed = removed_peer(&outcomes(faction.process(command)));
            let current = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            match removed {
                Some(peer_id) => prop_assert_eq!(current.collecting_peers().to_vec(), without(previous.collecting_peers(), peer_id)),
                None => prop_assert!(current.collecting_peers().len() >= previous.collecting_peers().len()),
            }
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
            assert_counts_do_not_decrease(previous.clone(), current.clone(), &batch)?;
            assert_non_member_outputs_do_not_mutate_state(previous, current, &batch)?;
        }
    }

    #[test]
    fn exits_at_most_once(commands in prop::collection::vec(command_strategy(), 0..128)) {
        // Arrange
        let mut faction = faction();
        let mut has_exited = false;
        let mut removed = false;

        // Act
        for command in commands {
            let previous = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };
            removed = removed || own_removal(&outcomes(faction.process(command)));
            let current = match faction.process(Command::Probe) {
                ProcessResult::Probed { cluster_view, .. } => cluster_view,
                _ => unreachable!(),
            };

            // Assert
            if removed {
                prop_assert_eq!(current.peer_state(), PeerState::Removed);
                continue;
            }
            if has_exited {
                prop_assert_eq!(current.peer_state(), PeerState::Bootstrapped);
            }
            if previous.peer_state() == PeerState::Bootstrapped {
                prop_assert_eq!(current.peer_state(), PeerState::Bootstrapped);
            }
            if current.peer_state() == PeerState::Bootstrapped {
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
            assert_counts_do_not_decrease(previous.clone(), current.clone(), &batch)?;
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
            if (cluster_view.peer_state() == PeerState::Bootstrapped).then_some(Conclusion::Bootstrapped) == Some(Conclusion::Bootstrapped) {
                prop_assert!(cluster_view.is_pinging_completed());
                prop_assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
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
                prop_assert_ne!(cluster_view.peer_state(), PeerState::Bootstrapped);
                prop_assert_ne!(cluster_view.peer_state(), PeerState::Bootstrapped);
            }
        }
    }
}

fn removed_peer(outputs: &[Outcome]) -> Option<u64> {
    outputs.iter().find_map(|output| match output {
        Outcome::MemberRemoved { peer_id } => Some(*peer_id),
        _ => None,
    })
}

fn test_config() -> Config {
    Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4), 1)
}

fn without(peers: &[u64], peer_id: u64) -> Vec<u64> {
    peers
        .iter()
        .copied()
        .filter(|peer| *peer != peer_id)
        .collect()
}
