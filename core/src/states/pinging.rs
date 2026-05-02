// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::conclusion::Conclusion;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::state::State;
use crate::PeerId;

use super::bootstrapped::Bootstrapped;
use super::collecting::Collecting;
use super::collecting_step::CollectingStep;
use super::local_completion_step::LocalCompletionStep;
use super::pinging_step::PingingStep;
use super::timed_out::TimedOut;

#[derive(Default)]
pub struct Pinging {
    pinging_peers: Vec<PeerId>,
    collecting_peers: Vec<PeerId>,
}

impl Pinging {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn compute_new_state(&self, is_quorum: bool, confirmed_peers: Vec<PeerId>) -> Box<dyn State> {
        if is_quorum {
            Box::new(Bootstrapped {
                pinged_peers: self.pinging_peers.clone(),
                collected_peers: confirmed_peers,
            })
        } else {
            Box::new(Collecting {
                collecting_peers: confirmed_peers,
                pinged_peers: self.pinging_peers.clone(),
            })
        }
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
            .with_pinging_peers(self.pinging_peers.clone())
            .with_collecting_peers(self.collecting_peers.clone())
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        if let Some(peer_id) = Self::non_member_peer(&command, config) {
            return (
                vec![Outcome::NonMemberIgnored { peer_id }],
                Box::new(Self {
                    pinging_peers: self.pinging_peers.clone(),
                    collecting_peers: self.collecting_peers.clone(),
                }),
            );
        }

        match command {
            Command::ParticipationObserved { peer_id } => {
                let step = PingingStep::new(self.pinging_peers.clone(), peer_id);

                (
                    step.outcomes().to_vec(),
                    Box::new(Self {
                        pinging_peers: step.confirmed_peers().to_vec(),
                        collecting_peers: self.collecting_peers.clone(),
                    }),
                )
            }

            Command::ReadyObserved { peer_id } => {
                let step = CollectingStep::new(self.collecting_peers.clone(), peer_id, None);

                (
                    step.outcomes().to_vec(),
                    Box::new(Self {
                        pinging_peers: self.pinging_peers.clone(),
                        collecting_peers: step.confirmed_peers().to_vec(),
                    }),
                )
            }

            Command::LocalParticipationCompleted => {
                let step = LocalCompletionStep::new(
                    self.collecting_peers.clone(),
                    config.peer_id(),
                    config.required_count(),
                );

                (
                    step.outcomes().to_vec(),
                    self.compute_new_state(step.is_quorum(), step.confirmed_peers().to_vec()),
                )
            }

            Command::DeadlineExpired => (
                vec![Outcome::Concluded {
                    mode: Conclusion::TimedOut,
                }],
                Box::new(TimedOut {
                    pinging_peers: self.pinging_peers.clone(),
                    collecting_peers: self.collecting_peers.clone(),
                }),
            ),

            Command::Probe => unreachable!("Probe handled in Faction::process"),
        }
    }
}
