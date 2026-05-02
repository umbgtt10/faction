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
use super::observed_step::ObservedKind;
use super::observed_step::ObservedStep;
use super::timed_out::TimedOut;

#[derive(Default)]
pub struct Collecting {
    pub collecting_peers: Vec<PeerId>,
    pub pinged_peers: Vec<PeerId>,
}

impl Collecting {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn non_member_peer(command: &Command, config: &Config) -> Option<PeerId> {
        match command {
            Command::ReadyObserved { peer_id, .. } if !config.is_member(*peer_id) => Some(*peer_id),
            _ => None,
        }
    }
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
            Command::ReadyObserved { peer_id: 0 },
            Command::DeadlineExpired,
            Command::Probe,
        ]
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::Collecting)
            .with_is_pinging_completed(true)
            .with_pinging_peers(self.pinged_peers.clone())
            .with_collecting_peers(self.collecting_peers.clone())
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        if let Some(peer_id) = Self::non_member_peer(&command, config) {
            return (
                vec![Outcome::NonMemberIgnored { peer_id }],
                Box::new(Self {
                    collecting_peers: self.collecting_peers.clone(),
                    pinged_peers: self.pinged_peers.clone(),
                }),
            );
        }

        match command {
            Command::ParticipationObserved { .. } => {
                unreachable!("accept() rejects this command for Collecting")
            }

            Command::ReadyObserved { peer_id } => {
                let step = ObservedStep::new(
                    self.collecting_peers.clone(),
                    peer_id,
                    ObservedKind::Ready,
                    Some(config.required_count()),
                );

                let new_state: Box<dyn State> = if step.is_quorum() {
                    Box::new(Bootstrapped {
                        collected_peers: step.confirmed_peers().to_vec(),
                        pinged_peers: self.pinged_peers.clone(),
                    })
                } else {
                    Box::new(Self {
                        collecting_peers: step.confirmed_peers().to_vec(),
                        pinged_peers: self.pinged_peers.clone(),
                    })
                };

                (step.outcomes().to_vec(), new_state)
            }

            Command::LocalParticipationCompleted => {
                unreachable!("accept() rejects this command for Collecting")
            }

            Command::DeadlineExpired => (
                vec![Outcome::Concluded {
                    mode: Conclusion::TimedOut,
                }],
                Box::new(TimedOut {
                    collecting_peers: self.collecting_peers.clone(),
                    pinging_peers: self.pinged_peers.clone(),
                }),
            ),

            Command::Probe => {
                unreachable!("Probe handled in Faction::process")
            }
        }
    }
}
