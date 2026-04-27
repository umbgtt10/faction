// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::vibe_config::VibeConfig;
use crate::vibe_input::VibeInput;
use crate::vibe_output::VibeOutput;
use crate::vibe_snapshot::VibeSnapshot;
use crate::vibe_state::VibeState;

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

impl VibeState for Pinging {
    fn deal(&self, _input: &VibeInput) -> bool {
        true
    }

    fn punch(
        self: Box<Self>,
        input: VibeInput,
        config: &VibeConfig,
    ) -> (Vec<VibeOutput>, Box<dyn VibeState>) {
        let Self { phase1, phase2 } = *self;

        match input {
            VibeInput::ParticipationObserved {
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

                let outputs = ObservedOutput::new(ObservedKind::Participation, peer_id)
                    .compute_output(index, classification, is_dup);
                let (phase1, _) = phase1.try_confirm(index, is_dup, classification);

                (outputs, Box::new(Self { phase1, phase2 }))
            }

            VibeInput::ReadyObserved {
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

                let outputs = ObservedOutput::new(ObservedKind::Ready, peer_id).compute_output(
                    index,
                    classification,
                    is_dup,
                );
                let (phase2, _) = phase2.try_confirm(index, is_dup, classification);

                (outputs, Box::new(Self { phase1, phase2 }))
            }

            VibeInput::LocalParticipationCompleted => {
                let local_index = config
                    .peer_index(config.local_peer_id())
                    .expect("local peer must be in peer set");
                let (phase2, _) = phase2.confirm(local_index);

                let mut outputs = vec![
                    VibeOutput::LocalParticipationCompleted,
                    VibeOutput::BroadcastLocalReady,
                ];

                let quorum = phase2.count() >= config.quorum_threshold();
                if quorum {
                    outputs.push(VibeOutput::ReadyQuorumReached);
                    outputs.push(VibeOutput::ReadinessExited {
                        mode: ReadinessExitMode::Quorum,
                    });
                }

                let new_state: Box<dyn VibeState> = if quorum {
                    Box::new(ReadyByQuorum { phase1, phase2 })
                } else {
                    Box::new(Collecting { phase1, phase2 })
                };
                (outputs, new_state)
            }

            VibeInput::DeadlineExpired => (
                vec![VibeOutput::ReadinessExited {
                    mode: ReadinessExitMode::Deadline,
                }],
                Box::new(ReadyByDeadline {
                    phase1,
                    phase2,
                    local_participation_complete: false,
                }),
            ),
        }
    }

    fn vibe_check(&self, quorum_threshold: usize) -> VibeSnapshot {
        VibeSnapshot::new(
            ReadinessLifecycleState::Phase1Active,
            None,
            false,
            false,
            self.phase1.count(),
            self.phase2.count(),
            quorum_threshold,
        )
    }
}
