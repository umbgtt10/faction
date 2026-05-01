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
    pub collecting_count: ConfirmedSet,
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

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::Collecting)
            .with_is_pinging_completed(true)
            .with_collecting_peers(self.collecting_count.confirmed_peers().to_vec())
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let collecting_count = self.collecting_count.clone();
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
                let is_member = config.is_member(peer_id);
                let classification = if is_member {
                    Some(
                        config
                            .freshness_policy()
                            .classify(current_marker, freshness),
                    )
                } else {
                    None
                };
                let is_dup = is_member && collecting_count.is_confirmed(peer_id);

                let output = ObservedOutput::new(ObservedKind::Ready, peer_id).compute_output(
                    is_member,
                    classification,
                    is_dup,
                );

                let (new_collecting_count, confirmed_new) =
                    collecting_count.try_confirm(peer_id, is_member, classification);

                let quorum =
                    confirmed_new && new_collecting_count.count() >= config.required_count();
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
                        collecting_count: new_collecting_count.count(),
                    })
                } else {
                    Box::new(Self {
                        collecting_count: new_collecting_count,
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
                    collecting_count: collecting_count.count(),
                }),
            ),

            Command::Probe => {
                unreachable!("Probe handled in Faction::process")
            }
        }
    }
}
