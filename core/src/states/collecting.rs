// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::machine_config::MachineConfig;
use crate::machine_input::MachineInput;
use crate::machine_output::MachineOutput;
use crate::machine_snapshot::MachineSnapshot;
use crate::machine_state::MachineState;

use super::helpers::compute_output::ObservedKind;
use super::helpers::compute_output::ObservedOutput;
use super::helpers::confirmed_set::ConfirmedSet;
use super::ready_by_deadline::ReadyByDeadline;
use super::ready_by_quorum::ReadyByQuorum;

pub struct Collecting {
    pub phase1: ConfirmedSet,
    pub phase2: ConfirmedSet,
}

impl MachineState for Collecting {
    fn accept(&self, input: &MachineInput) -> bool {
        matches!(
            input,
            MachineInput::ReadyObserved { .. } | MachineInput::DeadlineExpired
        )
    }

    fn step(
        self: Box<Self>,
        input: MachineInput,
        config: &MachineConfig,
    ) -> (Vec<MachineOutput>, Box<dyn MachineState>) {
        let Self { phase1, phase2 } = *self;

        match input {
            MachineInput::ParticipationObserved { .. } => {
                unreachable!("accept() rejects this input for Collecting")
            }

            MachineInput::ReadyObserved {
                peer_id,
                freshness,
                current_marker,
            } => {
                let index = config.peer_index(peer_id);
                let classification = index.map(|_| {
                    config
                        .freshness_policy()
                        .classify(current_marker, freshness)
                });
                let is_dup = index.is_some_and(|i| phase2.is_confirmed(i));

                let output = ObservedOutput::new(ObservedKind::Ready, peer_id).compute_output(
                    index,
                    classification,
                    is_dup,
                );

                let (phase2, confirmed_new) = phase2.try_confirm(index, is_dup, classification);

                let quorum = confirmed_new && phase2.count() >= config.quorum_threshold();
                let outputs = if quorum {
                    vec![
                        output,
                        MachineOutput::ReadyQuorumReached,
                        MachineOutput::ReadinessExited {
                            mode: ReadinessExitMode::Quorum,
                        },
                    ]
                } else {
                    vec![output]
                };

                let new_state: Box<dyn MachineState> = if quorum {
                    Box::new(ReadyByQuorum { phase1, phase2 })
                } else {
                    Box::new(Self { phase1, phase2 })
                };
                (outputs, new_state)
            }

            MachineInput::LocalParticipationCompleted => {
                unreachable!("accept() rejects this input for Collecting")
            }

            MachineInput::DeadlineExpired => (
                vec![MachineOutput::ReadinessExited {
                    mode: ReadinessExitMode::Deadline,
                }],
                Box::new(ReadyByDeadline {
                    phase1,
                    phase2,
                    local_participation_complete: true,
                }),
            ),
        }
    }

    fn snapshot(&self, quorum_threshold: usize) -> MachineSnapshot {
        MachineSnapshot::new(
            ReadinessLifecycleState::Phase2Active,
            None,
            true,
            false,
            self.phase1.count(),
            self.phase2.count(),
            quorum_threshold,
        )
    }
}
