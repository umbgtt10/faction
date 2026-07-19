// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::state::State;
use crate::types::PeerId;

use super::bootstrapped::Bootstrapped;
use super::ready_step::ReadyStep;

#[derive(Default)]
pub struct Collecting {
    collecting_peers: Vec<PeerId>,
    pinged_peers: Vec<PeerId>,
    deadline_missed: bool,
}

impl Collecting {
    #[must_use]
    pub fn new(
        collecting_peers: Vec<PeerId>,
        pinged_peers: Vec<PeerId>,
        deadline_missed: bool,
    ) -> Self {
        Self {
            collecting_peers,
            pinged_peers,
            deadline_missed,
        }
    }

    fn compute_new_state(&self, is_quorum: bool, confirmed_peers: Vec<PeerId>) -> Box<dyn State> {
        if is_quorum {
            Box::new(Bootstrapped::new(
                self.pinged_peers.clone(),
                confirmed_peers,
            ))
        } else {
            Box::new(Self::new(
                confirmed_peers,
                self.pinged_peers.clone(),
                self.deadline_missed,
            ))
        }
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
            .with_deadline_missed(self.deadline_missed)
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        if let Some(peer_id) = Self::non_member_peer(&command, config) {
            return (
                vec![Outcome::NonMemberIgnored { peer_id }],
                Box::new(Self::new(
                    self.collecting_peers.clone(),
                    self.pinged_peers.clone(),
                    self.deadline_missed,
                )),
            );
        }

        match command {
            Command::ParticipationObserved { .. } => {
                unreachable!("accept() rejects this command for Collecting")
            }

            Command::ReadyObserved { peer_id } => {
                let step = ReadyStep::new(
                    self.collecting_peers.clone(),
                    peer_id,
                    config.required_count(),
                );

                (
                    step.outcomes().to_vec(),
                    self.compute_new_state(step.is_quorum(), step.confirmed_peers().to_vec()),
                )
            }

            Command::LocalParticipationCompleted => {
                unreachable!("accept() rejects this command for Collecting")
            }

            Command::DeadlineExpired => (
                vec![Outcome::DeadlineMissed {
                    confirmed_count: self.collecting_peers.len(),
                }],
                Box::new(Self::new(
                    self.collecting_peers.clone(),
                    self.pinged_peers.clone(),
                    true,
                )),
            ),

            Command::Probe => {
                unreachable!("Probe handled in Faction::process")
            }
        }
    }
}
