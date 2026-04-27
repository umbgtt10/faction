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

use super::helpers::compute_output::ObservedKind;
use super::helpers::compute_output::ObservedOutput;
use super::helpers::confirmed_set::ConfirmedSet;
use super::ready_by_deadline::ReadyByDeadline;
use super::ready_by_quorum::ReadyByQuorum;

pub struct Collecting {
    pub phase1: ConfirmedSet,
    pub phase2: ConfirmedSet,
}

impl VibeState for Collecting {
    fn deal(&self, input: &VibeInput) -> bool {
        matches!(
            input,
            VibeInput::ReadyObserved { .. } | VibeInput::DeadlineExpired
        )
    }

    fn punch(
        self: Box<Self>,
        input: VibeInput,
        config: &VibeConfig,
    ) -> (Vec<VibeOutput>, Box<dyn VibeState>) {
        let Self { phase1, phase2 } = *self;

        match input {
            VibeInput::ParticipationObserved { .. } => {
                unreachable!("deal() rejects this input for Collecting")
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
                        VibeOutput::ReadyQuorumReached,
                        VibeOutput::ReadinessExited {
                            mode: ReadinessExitMode::Quorum,
                        },
                    ]
                } else {
                    vec![output]
                };

                let new_state: Box<dyn VibeState> = if quorum {
                    Box::new(ReadyByQuorum { phase1, phase2 })
                } else {
                    Box::new(Self { phase1, phase2 })
                };
                (outputs, new_state)
            }

            VibeInput::LocalParticipationCompleted => {
                unreachable!("deal() rejects this input for Collecting")
            }

            VibeInput::DeadlineExpired => (
                vec![VibeOutput::ReadinessExited {
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

    fn vibe_check(&self, quorum_threshold: usize) -> VibeSnapshot {
        VibeSnapshot::new(
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
