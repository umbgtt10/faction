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

use super::phase2::Phase2;
use super::ready_by_deadline::ReadyByDeadline;
use super::ready_by_quorum::ReadyByQuorum;

pub struct Phase1 {
    phase1_confirmed: Vec<bool>,
    phase2_confirmed: Vec<bool>,
    phase1_confirmed_count: usize,
    phase2_confirmed_count: usize,
}

impl Phase1 {
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

impl VibeState for Phase1 {
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
                if !config.is_member(peer_id) {
                    return (
                        vec![VibeOutput::NonMemberIgnored { peer_id }],
                        Box::new(Self {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    );
                }

                let classification = config
                    .freshness_policy()
                    .classify(current_marker, freshness);

                if classification == FreshnessClassification::Stale {
                    return (
                        vec![VibeOutput::StaleParticipationIgnored { peer_id }],
                        Box::new(Self {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    );
                }

                let Some(index) = config.peer_index(peer_id) else {
                    return (
                        vec![VibeOutput::NonMemberIgnored { peer_id }],
                        Box::new(Self {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    );
                };

                if phase1_confirmed[index] {
                    return (
                        vec![VibeOutput::DuplicateParticipationIgnored { peer_id }],
                        Box::new(Self {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    );
                }

                phase1_confirmed[index] = true;
                phase1_confirmed_count += 1;

                let outputs = if matches!(classification, FreshnessClassification::Timely) {
                    vec![VibeOutput::ParticipationAccepted { peer_id }]
                } else {
                    vec![VibeOutput::DelayedParticipationAccepted { peer_id }]
                };

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
                if !config.is_member(peer_id) {
                    return (
                        vec![VibeOutput::NonMemberIgnored { peer_id }],
                        Box::new(Self {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    );
                }

                let classification = config
                    .freshness_policy()
                    .classify(current_marker, freshness);

                if classification == FreshnessClassification::Stale {
                    return (
                        vec![VibeOutput::StaleReadyIgnored { peer_id }],
                        Box::new(Self {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    );
                }

                let Some(index) = config.peer_index(peer_id) else {
                    return (
                        vec![VibeOutput::NonMemberIgnored { peer_id }],
                        Box::new(Self {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    );
                };

                if phase2_confirmed[index] {
                    return (
                        vec![VibeOutput::DuplicateReadyIgnored { peer_id }],
                        Box::new(Self {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    );
                }

                phase2_confirmed[index] = true;
                phase2_confirmed_count += 1;

                let outputs = if matches!(classification, FreshnessClassification::Timely) {
                    vec![VibeOutput::ReadyAccepted { peer_id }]
                } else {
                    vec![VibeOutput::DelayedReadyAccepted { peer_id }]
                };

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

                if !phase2_confirmed[local_index] {
                    phase2_confirmed[local_index] = true;
                    phase2_confirmed_count += 1;
                }

                let outputs = vec![
                    VibeOutput::LocalParticipationCompleted,
                    VibeOutput::BroadcastLocalReady,
                ];

                if phase2_confirmed_count >= config.quorum_threshold() {
                    let mut emitted = outputs;
                    emitted.push(VibeOutput::ReadyQuorumReached);
                    emitted.push(VibeOutput::ReadinessExited {
                        mode: ReadinessExitMode::Quorum,
                    });
                    (
                        emitted,
                        Box::new(ReadyByQuorum {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    )
                } else {
                    (
                        outputs,
                        Box::new(Phase2 {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    )
                }
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
