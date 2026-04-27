// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::freshness_classification::FreshnessClassification;
use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::vibe_config::VibeConfig;
use crate::vibe_input::VibeInput;
use crate::vibe_output::VibeOutput;
use crate::vibe_snapshot::VibeSnapshot;
use crate::vibe_state::VibeState;

use super::helpers::compute_output;
use super::ready_by_deadline::ReadyByDeadline;
use super::ready_by_quorum::ReadyByQuorum;

pub struct Collecting {
    pub(super) phase1_confirmed: Vec<bool>,
    pub(super) phase2_confirmed: Vec<bool>,
    pub(super) phase1_confirmed_count: usize,
    pub(super) phase2_confirmed_count: usize,
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
        let Self {
            phase1_confirmed,
            mut phase2_confirmed,
            phase1_confirmed_count,
            mut phase2_confirmed_count,
        } = *self;

        match input {
            VibeInput::ParticipationObserved { .. } => (
                Vec::new(),
                Box::new(Self {
                    phase1_confirmed,
                    phase2_confirmed,
                    phase1_confirmed_count,
                    phase2_confirmed_count,
                }),
            ),

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
                let is_dup = index.is_some_and(|i| phase2_confirmed[i]);

                let calc = compute_output::ObservedOutput::new(
                    compute_output::ObservedKind::Ready,
                    peer_id,
                );
                let outputs = calc.compute_output(index, classification, is_dup);

                let (confirmed_new, new_phase2_confirmed, new_phase2_confirmed_count) =
                    match (index, is_dup, classification) {
                        (Some(i), false, Some(c)) if c != FreshnessClassification::Stale => (
                            true,
                            phase2_confirmed
                                .iter()
                                .enumerate()
                                .map(|(idx, &v)| if idx == i { true } else { v })
                                .collect(),
                            phase2_confirmed_count + 1,
                        ),
                        _ => (false, phase2_confirmed, phase2_confirmed_count),
                    };

                let quorum =
                    confirmed_new && new_phase2_confirmed_count >= config.quorum_threshold();
                let outputs = if quorum {
                    vec![
                        outputs[0],
                        VibeOutput::ReadyQuorumReached,
                        VibeOutput::ReadinessExited {
                            mode: ReadinessExitMode::Quorum,
                        },
                    ]
                } else {
                    outputs
                };

                phase2_confirmed = new_phase2_confirmed;
                phase2_confirmed_count = new_phase2_confirmed_count;

                let new_state: Box<dyn VibeState> = if quorum {
                    Box::new(ReadyByQuorum {
                        phase1_confirmed,
                        phase2_confirmed,
                        phase1_confirmed_count,
                        phase2_confirmed_count,
                    })
                } else {
                    Box::new(Self {
                        phase1_confirmed,
                        phase2_confirmed,
                        phase1_confirmed_count,
                        phase2_confirmed_count,
                    })
                };
                (outputs, new_state)
            }

            VibeInput::LocalParticipationCompleted => (
                Vec::new(),
                Box::new(Self {
                    phase1_confirmed,
                    phase2_confirmed,
                    phase1_confirmed_count,
                    phase2_confirmed_count,
                }),
            ),

            VibeInput::DeadlineExpired => (
                vec![VibeOutput::ReadinessExited {
                    mode: ReadinessExitMode::Deadline,
                }],
                Box::new(ReadyByDeadline {
                    phase1_confirmed,
                    phase2_confirmed,
                    phase1_confirmed_count,
                    phase2_confirmed_count,
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
            self.phase1_confirmed_count,
            self.phase2_confirmed_count,
            quorum_threshold,
        )
    }
}
