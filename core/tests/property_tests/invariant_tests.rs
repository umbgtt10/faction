// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::snapshot::Snapshot;
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
    previous: Snapshot,
    current: Snapshot,
) -> Result<(), TestCaseError> {
    prop_assert!(current.phase1_confirmed_count() >= previous.phase1_confirmed_count());
    prop_assert!(current.phase2_confirmed_count() >= previous.phase2_confirmed_count());
    Ok(())
}

fn assert_stale_outputs_do_not_mutate_state(
    previous: Snapshot,
    current: Snapshot,
    outputs: &[Outcome],
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
    previous: Snapshot,
    current: Snapshot,
    outputs: &[Outcome],
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
    previous: Snapshot,
    current: Snapshot,
    outputs: &[Outcome],
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
            let _ = coordinator.process(input);
            let snapshot = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

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
            let _ = coordinator.process(input);
            let snapshot = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

            // Assert
            if snapshot.readiness_exited() {
                has_exited = true;
                prop_assert!(matches!(
                    snapshot.lifecycle_state(),
                    ReadinessLifecycleState::Bootstrapped | ReadinessLifecycleState::TimedOut
                ));
            }

            if has_exited {
                prop_assert!(snapshot.readiness_exited());
                prop_assert!(matches!(
                    snapshot.lifecycle_state(),
                    ReadinessLifecycleState::Bootstrapped | ReadinessLifecycleState::TimedOut
                ));
            }
        }
    }

    #[test]
    fn phase1_count_never_decreases(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut previous = match coordinator.process(Command::Probe) {
    ProcessResult::Probed { snapshot, .. } => snapshot,
    _ => unreachable!(),
};

        // Act
        for input in inputs {
            let _ = coordinator.process(input);
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

            // Assert
            prop_assert!(current.phase1_confirmed_count() >= previous.phase1_confirmed_count());
            previous = current;
        }
    }

    #[test]
    fn phase2_count_never_decreases(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut previous = match coordinator.process(Command::Probe) {
    ProcessResult::Probed { snapshot, .. } => snapshot,
    _ => unreachable!(),
};

        // Act
        for input in inputs {
            let _ = coordinator.process(input);
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

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
            let previous = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };
            let batch = match coordinator.process(input) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

            // Assert
            assert_counts_do_not_decrease(previous, current)?;
            assert_stale_outputs_do_not_mutate_state(previous, current, &batch)?;
        }
    }

    #[test]
    fn non_member_inputs_never_mutate_counts(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let previous = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };
            let batch = match coordinator.process(input) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

            // Assert
            assert_counts_do_not_decrease(previous, current)?;
            assert_non_member_outputs_do_not_mutate_state(previous, current, &batch)?;
        }
    }

    #[test]
    fn readiness_exits_at_most_once(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();
        let mut has_exited = false;

        // Act
        for input in inputs {
            let previous = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };
            let _ = coordinator.process(input);
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

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
            let previous = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };
            let batch = match coordinator.process(input) {
                ProcessResult::Accepted { outcomes, .. } => outcomes,
                ProcessResult::Probed { .. } => unreachable!(),
                ProcessResult::Rejected { .. } => vec![],
            };
            let current = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

            // Assert
            assert_counts_do_not_decrease(previous, current)?;
            assert_duplicate_outputs_do_not_mutate_counts(previous, current, &batch)?;
        }
    }

    #[test]
    fn quorum_exit_implies_local_participation_completion(inputs in prop::collection::vec(input_strategy(), 0..128)) {
        // Arrange
        let mut coordinator = coordinator();

        // Act
        for input in inputs {
            let _ = coordinator.process(input);
            let snapshot = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

            // Assert
            if snapshot.exit_mode() == Some(ReadinessExitMode::Bootstrapped) {
                prop_assert!(snapshot.local_participation_complete());
                prop_assert!(snapshot.readiness_exited());
                prop_assert_eq!(
                    snapshot.lifecycle_state(),
                    ReadinessLifecycleState::Bootstrapped
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
            let _ = coordinator.process(input);
            let snapshot = match coordinator.process(Command::Probe) {
                ProcessResult::Probed { snapshot, .. } => snapshot,
                _ => unreachable!(),
            };

            // Assert
            if snapshot.exit_mode() == Some(ReadinessExitMode::TimedOut) {
                prop_assert!(snapshot.readiness_exited());
                prop_assert_eq!(
                    snapshot.lifecycle_state(),
                    ReadinessLifecycleState::TimedOut
                );
            }
        }
    }
}
