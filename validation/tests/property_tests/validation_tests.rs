// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::outcome::Outcome;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction_validation::scenario_harness::ScenarioHarness;
use proptest::prelude::*;

#[derive(Debug, Clone, Copy)]
enum ScenarioOperation {
    Participation {
        coordinator_index: usize,
        peer_id: u64,
        freshness: u64,
    },
    Ready {
        coordinator_index: usize,
        peer_id: u64,
        freshness: u64,
    },
    CompleteLocalParticipation {
        coordinator_index: usize,
    },
    ExpireDeadline {
        coordinator_index: usize,
    },
    AdvanceTo {
        marker: u64,
    },
    AdvanceBy {
        delta: u64,
    },
}

fn operation_strategy() -> impl Strategy<Value = ScenarioOperation> {
    prop_oneof![
        (0usize..5, 0u64..=6, 0u64..=12).prop_map(|(coordinator_index, peer_id, freshness)| {
            ScenarioOperation::Participation {
                coordinator_index,
                peer_id,
                freshness,
            }
        }),
        (0usize..5, 0u64..=6, 0u64..=12).prop_map(|(coordinator_index, peer_id, freshness)| {
            ScenarioOperation::Ready {
                coordinator_index,
                peer_id,
                freshness,
            }
        }),
        (0usize..5).prop_map(|coordinator_index| {
            ScenarioOperation::CompleteLocalParticipation { coordinator_index }
        }),
        (0usize..5).prop_map(|coordinator_index| {
            ScenarioOperation::ExpireDeadline { coordinator_index }
        }),
        (0u64..=12).prop_map(|marker| ScenarioOperation::AdvanceTo { marker }),
        (0u64..=3).prop_map(|delta| ScenarioOperation::AdvanceBy { delta }),
    ]
}

fn harness() -> ScenarioHarness {
    ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2)
}

fn outputs_contain_duplicate(outputs: &[Outcome]) -> bool {
    outputs.iter().any(|output| {
        matches!(
            output,
            Outcome::DuplicateParticipationIgnored { .. } | Outcome::DuplicateReadyIgnored { .. }
        )
    })
}

fn outputs_contain_stale(outputs: &[Outcome]) -> bool {
    outputs.iter().any(|output| {
        matches!(
            output,
            Outcome::StaleParticipationIgnored { .. } | Outcome::StaleReadyIgnored { .. }
        )
    })
}

fn apply_operation(
    harness: &mut ScenarioHarness,
    operation: ScenarioOperation,
) -> Option<(usize, Vec<Outcome>)> {
    match operation {
        ScenarioOperation::Participation {
            coordinator_index,
            peer_id,
            freshness,
        } => Some((
            coordinator_index,
            harness.apply_participation(coordinator_index, peer_id, freshness),
        )),
        ScenarioOperation::Ready {
            coordinator_index,
            peer_id,
            freshness,
        } => Some((
            coordinator_index,
            harness.apply_ready(coordinator_index, peer_id, freshness),
        )),
        ScenarioOperation::CompleteLocalParticipation { coordinator_index } => Some((
            coordinator_index,
            harness.complete_local_participation(coordinator_index),
        )),
        ScenarioOperation::ExpireDeadline { coordinator_index } => Some((
            coordinator_index,
            harness.expire_deadline(coordinator_index),
        )),
        ScenarioOperation::AdvanceTo { marker } => {
            harness.advance_to(marker);
            None
        }
        ScenarioOperation::AdvanceBy { delta } => {
            harness.advance_by(delta);
            None
        }
    }
}

proptest! {
    #[test]
    fn duplicate_signals_across_nodes_never_increase_counts_after_first_accept(
        operations in prop::collection::vec(operation_strategy(), 0..128)
    ) {
        // Arrange
        let mut harness = harness();

        // Act
        for operation in operations {
            let previous_snapshots = (0..5)
                .map(|index| harness.snapshot(index))
                .collect::<Vec<_>>();
            let result = apply_operation(&mut harness, operation);

            // Assert
            if let Some((coordinator_index, outputs)) = result {
                if outputs_contain_duplicate(&outputs) {
                    let previous = previous_snapshots[coordinator_index];
                    let current = harness.snapshot(coordinator_index);
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
            }
        }
    }

    #[test]
    fn stale_signals_across_nodes_never_mutate_state(
        operations in prop::collection::vec(operation_strategy(), 0..128)
    ) {
        // Arrange
        let mut harness = harness();

        // Act
        for operation in operations {
            let previous_snapshots = (0..5)
                .map(|index| harness.snapshot(index))
                .collect::<Vec<_>>();
            let result = apply_operation(&mut harness, operation);

            // Assert
            if let Some((coordinator_index, outputs)) = result {
                if outputs_contain_stale(&outputs) {
                    let previous = previous_snapshots[coordinator_index];
                    let current = harness.snapshot(coordinator_index);
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
            }
        }
    }

    #[test]
    fn once_a_coordinator_exits_it_never_reopens_under_any_later_signal_sequence(
        operations in prop::collection::vec(operation_strategy(), 0..128)
    ) {
        // Arrange
        let mut harness = harness();
        let mut exited = [false; 5];

        // Act
        for operation in operations {
            let _ = apply_operation(&mut harness, operation);

            // Assert
            for (index, has_exited) in exited.iter_mut().enumerate() {
                let snapshot = harness.snapshot(index);
                if snapshot.readiness_exited() {
                    *has_exited = true;
                    prop_assert!(matches!(
                        snapshot.lifecycle_state(),
                        ReadinessLifecycleState::ReadyByQuorum
                            | ReadinessLifecycleState::TimedOut
                    ));
                }

                if *has_exited {
                    prop_assert!(snapshot.readiness_exited());
                    prop_assert!(matches!(
                        snapshot.lifecycle_state(),
                        ReadinessLifecycleState::ReadyByQuorum
                            | ReadinessLifecycleState::TimedOut
                    ));
                }
            }
        }
    }
}
