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

use super::collecting::Collecting;
use super::compute;
use super::ready_by_deadline::ReadyByDeadline;
use super::ready_by_quorum::ReadyByQuorum;

pub struct Pinging {
    phase1_confirmed: Vec<bool>,
    phase2_confirmed: Vec<bool>,
    phase1_confirmed_count: usize,
    phase2_confirmed_count: usize,
}

impl Pinging {
    #[must_use]
    pub fn new(peer_count: usize) -> Self {
        Self {
            phase1_confirmed: vec![false; peer_count],
            phase2_confirmed: vec![false; peer_count],
            phase1_confirmed_count: 0,
            phase2_confirmed_count: 0,
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
        let Self {
            mut phase1_confirmed,
            mut phase2_confirmed,
            mut phase1_confirmed_count,
            mut phase2_confirmed_count,
        } = *self;

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
                let is_dup = index.is_some_and(|i| phase1_confirmed[i]);

                let outputs = compute::observed_output(
                    compute::ObservedKind::Participation,
                    peer_id,
                    index,
                    classification,
                    is_dup,
                );

                let (new_phase1_confirmed, new_phase1_confirmed_count) =
                    match (index, is_dup, classification) {
                        (Some(i), false, Some(c)) if c != FreshnessClassification::Stale => (
                            phase1_confirmed
                                .iter()
                                .enumerate()
                                .map(|(j, &val)| j == i || val)
                                .collect(),
                            phase1_confirmed_count + 1,
                        ),
                        _ => (phase1_confirmed, phase1_confirmed_count),
                    };
                phase1_confirmed = new_phase1_confirmed;
                phase1_confirmed_count = new_phase1_confirmed_count;

                (
                    outputs,
                    Box::new(Self {
                        phase1_confirmed,
                        phase2_confirmed,
                        phase1_confirmed_count,
                        phase2_confirmed_count,
                    }),
                )
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
                let is_dup = index.is_some_and(|i| phase2_confirmed[i]);

                let outputs = compute::observed_output(
                    compute::ObservedKind::Ready,
                    peer_id,
                    index,
                    classification,
                    is_dup,
                );

                let (new_phase2_confirmed, new_phase2_confirmed_count) =
                    match (index, is_dup, classification) {
                        (Some(i), false, Some(c)) if c != FreshnessClassification::Stale => (
                            phase2_confirmed
                                .iter()
                                .enumerate()
                                .map(|(j, &val)| j == i || val)
                                .collect(),
                            phase2_confirmed_count + 1,
                        ),
                        _ => (phase2_confirmed, phase2_confirmed_count),
                    };

                phase2_confirmed = new_phase2_confirmed;
                phase2_confirmed_count = new_phase2_confirmed_count;

                (
                    outputs,
                    Box::new(Self {
                        phase1_confirmed,
                        phase2_confirmed,
                        phase1_confirmed_count,
                        phase2_confirmed_count,
                    }),
                )
            }

            VibeInput::LocalParticipationCompleted => {
                let local_index = config
                    .peer_index(config.local_peer_id())
                    .expect("local peer must be in peer set");
                let already_confirmed = phase2_confirmed[local_index];

                let (new_phase2_confirmed, new_phase2_confirmed_count) = if already_confirmed {
                    (phase2_confirmed, phase2_confirmed_count)
                } else {
                    (
                        phase2_confirmed
                            .iter()
                            .enumerate()
                            .map(|(i, &v)| i == local_index || v)
                            .collect(),
                        phase2_confirmed_count + 1,
                    )
                };
                phase2_confirmed = new_phase2_confirmed;
                phase2_confirmed_count = new_phase2_confirmed_count;

                let outputs = vec![
                    VibeOutput::LocalParticipationCompleted,
                    VibeOutput::BroadcastLocalReady,
                ];

                let quorum = phase2_confirmed_count >= config.quorum_threshold();
                let outputs = if quorum {
                    let mut extended = outputs;
                    extended.push(VibeOutput::ReadyQuorumReached);
                    extended.push(VibeOutput::ReadinessExited {
                        mode: ReadinessExitMode::Quorum,
                    });
                    extended
                } else {
                    outputs
                };
                let new_state: Box<dyn VibeState> = if quorum {
                    Box::new(ReadyByQuorum {
                        phase1_confirmed,
                        phase2_confirmed,
                        phase1_confirmed_count,
                        phase2_confirmed_count,
                    })
                } else {
                    Box::new(Collecting {
                        phase1_confirmed,
                        phase2_confirmed,
                        phase1_confirmed_count,
                        phase2_confirmed_count,
                    })
                };
                (outputs, new_state)
            }

            VibeInput::DeadlineExpired => (
                vec![VibeOutput::ReadinessExited {
                    mode: ReadinessExitMode::Deadline,
                }],
                Box::new(ReadyByDeadline {
                    phase1_confirmed,
                    phase2_confirmed,
                    phase1_confirmed_count,
                    phase2_confirmed_count,
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
            self.phase1_confirmed_count,
            self.phase2_confirmed_count,
            quorum_threshold,
        )
    }
}
