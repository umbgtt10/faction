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
    pinged_peers: Vec<PeerId>,
    collected_peers: Vec<PeerId>,
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
            .with_pinging_peers(self.pinged_peers.clone())
            .with_collecting_peers(self.collected_peers.clone())
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        let pinged_peers = self.pinged_peers.clone();
        let mut new_collected_peers = self.collected_peers.clone();

        if let Some(peer_id) = Self::non_member_peer(&command, config) {
            return (
                vec![Outcome::NonMemberIgnored { peer_id }],
                Box::new(Self {
                    pinged_peers,
                    collected_peers: new_collected_peers,
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
                    pinged_peers,
                    peer_id,
                    ObservedKind::Participation,
                    None,
                );

                (
                    step.outputs(),
                    Box::new(Self {
                        pinged_peers: step.confirmed_peers(),
                        collected_peers: new_collected_peers,
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
                    new_collected_peers,
                    peer_id,
                    ObservedKind::Ready,
                    None,
                );

                (
                    step.outputs(),
                    Box::new(Self {
                        pinged_peers,
                        collected_peers: step.confirmed_peers(),
                    }),
                )
            }

            Command::LocalParticipationCompleted => {
                if !new_collected_peers.contains(&config.peer_id()) {
                    new_collected_peers.push(config.peer_id());
                }

                let mut outputs = vec![
                    Outcome::LocalParticipationCompleted,
                    Outcome::BroadcastLocalReady,
                ];

                let quorum = new_collected_peers.len() >= config.required_count();
                if quorum {
                    outputs.push(Outcome::ReadyQuorumReached);
                    outputs.push(Outcome::Exited {
                        mode: ExitMode::Bootstrapped,
                    });
                }

                let new_state: Box<dyn State> = if quorum {
                    Box::new(Bootstrapped {
                        pinged_peers_count: pinged_peers.len(),
                        collected_peers_count: new_collected_peers.len(),
                    })
                } else {
                    Box::new(Collecting {
                        collected_peers: new_collected_peers,
                        pinged_peers_count: pinged_peers.len(),
                    })
                };
                (outputs, new_state)
            }

            Command::DeadlineExpired => (
                vec![Outcome::Exited {
                    mode: ExitMode::TimedOut,
                }],
                Box::new(TimedOut {
                    pinged_peers_count: pinged_peers.len(),
                    collected_peers_count: new_collected_peers.len(),
                }),
            ),

            Command::Probe => unreachable!("Probe handled in Faction::process"),
        }
    }
}
