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

use super::collecting::Collecting;
use super::helpers::compute_output::ObservedKind;
use super::helpers::compute_output::ObservedOutput;
use super::helpers::confirmed_set::ConfirmedSet;
use super::ready_by_deadline::ReadyByDeadline;
use super::ready_by_quorum::ReadyByQuorum;

pub struct Pinging {
    phase1: ConfirmedSet,
    phase2: ConfirmedSet,
}

impl Pinging {
    #[must_use]
    pub fn new(peer_count: usize) -> Self {
        Self {
            phase1: ConfirmedSet::new(peer_count),
            phase2: ConfirmedSet::new(peer_count),
        }
    }
}

impl StateSnapshot for Pinging {
    fn state_snapshot(&self, previous: &MachineSnapshot) -> MachineSnapshot {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::Phase1Active)
            .with_phase1_count(self.phase1.count())
            .with_phase2_count(self.phase2.count())
    }
}

impl MachineState for Pinging {
    fn step(
        self: Box<Self>,
        input: MachineInput,
        config: &MachineConfig,
    ) -> (Vec<MachineOutput>, Box<dyn MachineState>) {
        let Self { phase1, phase2 } = *self;

        match input {
            MachineInput::ParticipationObserved {
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
                let is_dup = index.is_some_and(|i| phase1.is_confirmed(i));

                let output = ObservedOutput::new(ObservedKind::Participation, peer_id)
                    .compute_output(index, classification, is_dup);
                let (phase1, _) = phase1.try_confirm(index, is_dup, classification);

                (vec![output], Box::new(Self { phase1, phase2 }))
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
                let (phase2, _) = phase2.try_confirm(index, is_dup, classification);

                (vec![output], Box::new(Self { phase1, phase2 }))
            }

            MachineInput::LocalParticipationCompleted => {
                let local_index = config
                    .peer_index(config.local_peer_id())
                    .expect("local peer must be in peer set");
                let (phase2, _) = phase2.confirm(local_index);

                let mut outputs = vec![
                    MachineOutput::LocalParticipationCompleted,
                    MachineOutput::BroadcastLocalReady,
                ];

                let quorum = phase2.count() >= config.quorum_threshold();
                if quorum {
                    outputs.push(MachineOutput::ReadyQuorumReached);
                    outputs.push(MachineOutput::ReadinessExited {
                        mode: ReadinessExitMode::Quorum,
                    });
                }

                let new_state: Box<dyn MachineState> = if quorum {
                    Box::new(ReadyByQuorum {
                        phase1_count: phase1.count(),
                        phase2_count: phase2.count(),
                    })
                } else {
                    Box::new(Collecting {
                        phase2,
                        phase1_count: phase1.count(),
                    })
                };
                (outputs, new_state)
            }

            MachineInput::DeadlineExpired => (
                vec![MachineOutput::ReadinessExited {
                    mode: ReadinessExitMode::Deadline,
                }],
                Box::new(ReadyByDeadline {
                    phase1_count: phase1.count(),
                    phase2_count: phase2.count(),
                }),
            ),
        }
    }
}
