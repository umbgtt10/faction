// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::snapshot::Snapshot;
use crate::state::State;
use crate::state_snapshot::StateSnapshot;

use super::compute_output::ObservedKind;
use super::compute_output::ObservedOutput;
use super::confirmed_set::ConfirmedSet;
use super::ready_by_deadline::ReadyByDeadline;
use super::ready_by_quorum::ReadyByQuorum;

pub struct Collecting {
    pub phase2: ConfirmedSet,
    pub phase1_count: usize,
}

impl State for Collecting {
    fn accept(&self, input: &Command) -> bool {
        matches!(
            input,
            Command::ReadyObserved { .. } | Command::DeadlineExpired
        )
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![
            Command::ReadyObserved {
                peer_id: 0,
                freshness: 0,
                current_marker: 0,
            },
            Command::DeadlineExpired,
            Command::Probe,
        ]
    }

    fn step(&self, input: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let phase2 = self.phase2.clone();
        let phase1_count = self.phase1_count;

        match input {
            Command::ParticipationObserved { .. } => {
                unreachable!("accept() rejects this input for Collecting")
            }

            Command::ReadyObserved {
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
                        Outcome::ReadyQuorumReached,
                        Outcome::ReadinessExited {
                            mode: ReadinessExitMode::Quorum,
                        },
                    ]
                } else {
                    vec![output]
                };

                let new_state: Box<dyn State> = if quorum {
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

            Command::LocalParticipationCompleted => {
                unreachable!("accept() rejects this input for Collecting")
            }

            Command::DeadlineExpired => (
                vec![Outcome::ReadinessExited {
                    mode: ReadinessExitMode::Deadline,
                }],
                Box::new(ReadyByDeadline {
                    phase1_count,
                    phase2_count: phase2.count(),
                }),
            ),

            Command::Probe => {
                unreachable!("Probe handled in Faction::process")
            }
        }
    }
}

impl StateSnapshot for Collecting {
    fn state_snapshot(&self, previous: &Snapshot) -> Snapshot {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::Phase2Active)
            .with_local_participation_complete(true)
            .with_phase1_count(self.phase1_count)
            .with_phase2_count(self.phase2.count())
    }
}
