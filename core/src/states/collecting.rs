// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::machine_config::MachineConfig;
use crate::machine_input::MachineInput;
use crate::machine_output::MachineOutput;
use crate::machine_snapshot::MachineSnapshot;
use crate::machine_state::MachineState;
use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::state_snapshot::StateSnapshot;

use super::helpers::compute_output::ObservedKind;
use super::helpers::compute_output::ObservedOutput;
use super::helpers::confirmed_set::ConfirmedSet;
use super::ready_by_deadline::ReadyByDeadline;
use super::ready_by_quorum::ReadyByQuorum;

pub struct Collecting {
    pub phase2: ConfirmedSet,
    pub phase1_count: usize,
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
        let Self {
            phase2,
            phase1_count,
        } = *self;

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
                    Box::new(ReadyByQuorum {
                        phase1_count,
                        phase2_count: phase2.count(),
                    })
                } else {
                    Box::new(Self {
                        phase2,
                        phase1_count,
                    })
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
                    phase1_count,
                    phase2_count: phase2.count(),
                }),
            ),

            MachineInput::GetSnapshot => {
                unreachable!("GetSnapshot handled in Machine::apply")
            }
        }
    }
}

impl StateSnapshot for Collecting {
    fn state_snapshot(&self, previous: &MachineSnapshot) -> MachineSnapshot {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::Phase2Active)
            .with_local_participation_complete(true)
            .with_phase1_count(self.phase1_count)
            .with_phase2_count(self.phase2.count())
    }
}
