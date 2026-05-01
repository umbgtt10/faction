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
use crate::PeerId;

use super::bootstrapped::Bootstrapped;
use super::collecting::Collecting;
use super::compute_output::ObservedKind;
use super::observed_step::ObservedStep;
use super::timed_out::TimedOut;

#[derive(Default)]
pub struct Pinging {
    pinging_count: Vec<PeerId>,
    collecting_count: Vec<PeerId>,
}

impl Pinging {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn non_member_peer(command: &Command, config: &Config) -> Option<PeerId> {
        match command {
            Command::ParticipationObserved { peer_id, .. }
            | Command::ReadyObserved { peer_id, .. }
                if !config.is_member(*peer_id) =>
            {
                Some(*peer_id)
            }
            _ => None,
        }
    }
}

impl State for Pinging {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::Pinging)
            .with_pinging_peers(self.pinging_count.clone())
            .with_collecting_peers(self.collecting_count.clone())
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let pinging_count = self.pinging_count.clone();
        let mut new_collecting_count = self.collecting_count.clone();

        if let Some(peer_id) = Self::non_member_peer(&command, config) {
            return (
                vec![Outcome::NonMemberIgnored { peer_id }],
                Box::new(Self {
                    pinging_count,
                    collecting_count: new_collecting_count,
                }),
            );
        }

        match command {
            Command::ParticipationObserved {
                peer_id,
                freshness,
                current_marker,
            } => {
                let classification = config
                    .freshness_policy()
                    .classify(current_marker, freshness);
                let step = ObservedStep::new(
                    classification,
                    pinging_count,
                    peer_id,
                    ObservedKind::Participation,
                );

                (
                    vec![step.outcome()],
                    Box::new(Self {
                        pinging_count: step.confirmed_peers(),
                        collecting_count: new_collecting_count,
                    }),
                )
            }

            Command::ReadyObserved {
                peer_id,
                freshness,
                current_marker,
            } => {
                let classification = config
                    .freshness_policy()
                    .classify(current_marker, freshness);
                let step = ObservedStep::new(
                    classification,
                    new_collecting_count,
                    peer_id,
                    ObservedKind::Ready,
                );

                (
                    vec![step.outcome()],
                    Box::new(Self {
                        pinging_count,
                        collecting_count: step.confirmed_peers(),
                    }),
                )
            }

            Command::LocalParticipationCompleted => {
                if !new_collecting_count.contains(&config.peer_id()) {
                    new_collecting_count.push(config.peer_id());
                }

                let mut outputs = vec![
                    Outcome::LocalParticipationCompleted,
                    Outcome::BroadcastLocalReady,
                ];

                let quorum = new_collecting_count.len() >= config.required_count();
                if quorum {
                    outputs.push(Outcome::ReadyQuorumReached);
                    outputs.push(Outcome::Exited {
                        mode: ExitMode::Bootstrapped,
                    });
                }

                let new_state: Box<dyn State> = if quorum {
                    Box::new(Bootstrapped {
                        pinging_count: pinging_count.len(),
                        collecting_count: new_collecting_count.len(),
                    })
                } else {
                    Box::new(Collecting {
                        collecting_count: new_collecting_count,
                        pinging_count: pinging_count.len(),
                    })
                };
                (outputs, new_state)
            }

            Command::DeadlineExpired => (
                vec![Outcome::Exited {
                    mode: ExitMode::TimedOut,
                }],
                Box::new(TimedOut {
                    pinging_count: pinging_count.len(),
                    collecting_count: new_collecting_count.len(),
                }),
            ),

            Command::Probe => unreachable!("Probe handled in Faction::process"),
        }
    }
}
