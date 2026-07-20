// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec;

use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction_core_validation::scenario_harness::ScenarioHarness;
use proptest::prelude::*;

#[derive(Debug, Clone, Copy)]
enum ScenarioOperation {
    Participation {
        coordinator_index: usize,
        peer_id: u64,
    },
    Ready {
        coordinator_index: usize,
        peer_id: u64,
    },
    CompleteLocalParticipation {
        coordinator_index: usize,
    },
    ExpireDeadline {
        coordinator_index: usize,
    },
}

fn operation_strategy() -> impl Strategy<Value = ScenarioOperation> {
    prop_oneof![
        (0usize..5, 0u64..=6).prop_map(|(coordinator_index, peer_id)| {
            ScenarioOperation::Participation {
                coordinator_index,
                peer_id,
            }
        }),
        (0usize..5, 0u64..=6).prop_map(|(coordinator_index, peer_id)| {
            ScenarioOperation::Ready {
                coordinator_index,
                peer_id,
            }
        }),
        (0usize..5).prop_map(|coordinator_index| {
            ScenarioOperation::CompleteLocalParticipation { coordinator_index }
        }),
        (0usize..5).prop_map(|coordinator_index| {
            ScenarioOperation::ExpireDeadline { coordinator_index }
        }),
    ]
}

fn harness() -> ScenarioHarness {
    ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4)
}

fn outputs_contain_duplicate(outputs: &[Outcome]) -> bool {
    outputs.iter().any(|output| {
        matches!(
            output,
            Outcome::DuplicateParticipationIgnored { .. } | Outcome::DuplicateReadyIgnored { .. }
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
        } => Some((
            coordinator_index,
            harness.apply_participation(coordinator_index, peer_id),
        )),
        ScenarioOperation::Ready {
            coordinator_index,
            peer_id,
        } => Some((
            coordinator_index,
            harness.apply_ready(coordinator_index, peer_id),
        )),
        ScenarioOperation::CompleteLocalParticipation { coordinator_index } => Some((
            coordinator_index,
            harness.complete_local_participation(coordinator_index),
        )),
        ScenarioOperation::ExpireDeadline { coordinator_index } => Some((
            coordinator_index,
            harness.expire_deadline(coordinator_index),
        )),
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
                .map(|index| harness.cluster_view(index))
                .collect::<Vec<_>>();
            let result = apply_operation(&mut harness, operation);

            // Assert
            if let Some((coordinator_index, outputs)) = result {
                if outputs_contain_duplicate(&outputs) {
                    let previous = &previous_snapshots[coordinator_index];
                    let current = harness.cluster_view(coordinator_index);
                    prop_assert_eq!(
                        current.pinging_peers().len(),
                        previous.pinging_peers().len()
                    );
                    prop_assert_eq!(
                        current.collecting_peers().len(),
                        previous.collecting_peers().len()
                    );
                    prop_assert_eq!(current.peer_state(), previous.peer_state());
                    prop_assert_eq!(current.conclusion(), previous.conclusion());
                    prop_assert_eq!(
                        current.is_pinging_completed(),
                        previous.is_pinging_completed()
                    );
                    prop_assert_eq!(current.is_concluded(), previous.is_concluded());
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
                let cluster_view = harness.cluster_view(index);
                if cluster_view.is_concluded() {
                    *has_exited = true;
                    prop_assert!(matches!(
                        cluster_view.peer_state(),
                        PeerState::Bootstrapped
                            | PeerState::TimedOut
                    ));
                }

                if *has_exited {
                    prop_assert!(cluster_view.is_concluded());
                    prop_assert!(matches!(
                        cluster_view.peer_state(),
                        PeerState::Bootstrapped
                            | PeerState::TimedOut
                    ));
                }
            }
        }
    }
}
