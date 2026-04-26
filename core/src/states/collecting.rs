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

                let accepted_output = if matches!(classification, FreshnessClassification::Timely) {
                    VibeOutput::ReadyAccepted { peer_id }
                } else {
                    VibeOutput::DelayedReadyAccepted { peer_id }
                };

                if phase2_confirmed_count >= config.quorum_threshold() {
                    (
                        vec![
                            accepted_output,
                            VibeOutput::ReadyQuorumReached,
                            VibeOutput::ReadinessExited {
                                mode: ReadinessExitMode::Quorum,
                            },
                        ],
                        Box::new(ReadyByQuorum {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    )
                } else {
                    (
                        vec![accepted_output],
                        Box::new(Self {
                            phase1_confirmed,
                            phase2_confirmed,
                            phase1_confirmed_count,
                            phase2_confirmed_count,
                        }),
                    )
                }
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
