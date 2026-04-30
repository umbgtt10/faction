// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::exit_mode::ExitMode;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::state::State;

use super::bootstrapped::Bootstrapped;
use super::compute_output::ObservedKind;
use super::compute_output::ObservedOutput;
use super::confirmed_set::ConfirmedSet;
use super::timed_out::TimedOut;

pub struct Collecting {
    pub phase2: ConfirmedSet,
    pub pinging_count: usize,
}

impl State for Collecting {
    fn accept(&self, command: &Command) -> bool {
        matches!(
            command,
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

    fn cluster_view(&self, previous: &ClusterView, config: &Config) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::Collecting)
            .with_is_pinging_completed(true)
            .with_collecting_peers(self.phase2.confirmed_peers(config.peers()))
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let phase2 = self.phase2.clone();
        let pinging_count = self.pinging_count;

        match command {
            Command::ParticipationObserved { .. } => {
                unreachable!("accept() rejects this command for Collecting")
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

                let quorum = confirmed_new && phase2.count() >= config.required_count();
                let outputs = if quorum {
                    vec![
                        output,
                        Outcome::ReadyQuorumReached,
                        Outcome::Exited {
                            mode: ExitMode::Bootstrapped,
                        },
                    ]
                } else {
                    vec![output]
                };

                let new_state: Box<dyn State> = if quorum {
                    Box::new(Bootstrapped {
                        pinging_count,
                        collecting_count: phase2.count(),
                    })
                } else {
                    Box::new(Self {
                        phase2,
                        pinging_count,
                    })
                };
                (outputs, new_state)
            }

            Command::LocalParticipationCompleted => {
                unreachable!("accept() rejects this command for Collecting")
            }

            Command::DeadlineExpired => (
                vec![Outcome::Exited {
                    mode: ExitMode::TimedOut,
                }],
                Box::new(TimedOut {
                    pinging_count,
                    collecting_count: phase2.count(),
                }),
            ),

            Command::Probe => {
                unreachable!("Probe handled in Faction::process")
            }
        }
    }
}
