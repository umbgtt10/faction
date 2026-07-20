// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::state::State;
use crate::types::PeerId;

use super::bootstrapped::Bootstrapped;
use super::collecting::Collecting;
use super::join_step::JoinStep;
use super::local_completion_step::LocalCompletionStep;
use super::pinging_step::PingingStep;

pub struct Pinging {
    members: Members,
    pinging_peers: Vec<PeerId>,
    collecting_peers: Vec<PeerId>,
    deadline_missed: bool,
}

impl Pinging {
    #[must_use]
    pub fn new(members: Members) -> Self {
        Self {
            members,
            pinging_peers: Vec::new(),
            collecting_peers: Vec::new(),
            deadline_missed: false,
        }
    }

    fn compute_new_state(&self, is_quorum: bool, confirmed_peers: Vec<PeerId>) -> Box<dyn State> {
        if is_quorum {
            Box::new(Bootstrapped::new(
                self.members.clone(),
                self.pinging_peers.clone(),
                confirmed_peers,
            ))
        } else {
            Box::new(Collecting::new(
                self.members.clone(),
                confirmed_peers,
                self.pinging_peers.clone(),
                self.deadline_missed,
            ))
        }
    }

    fn non_member_peer(&self, command: &Command) -> Option<PeerId> {
        match command {
            Command::ParticipationObserved { peer_id, .. }
            | Command::ReadyObserved { peer_id, .. }
                if !self.members.is_member(*peer_id) =>
            {
                Some(*peer_id)
            }
            _ => None,
        }
    }
}

impl State for Pinging {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Pinging)
            .with_pinging_peers(self.pinging_peers.clone())
            .with_collecting_peers(self.collecting_peers.clone())
            .with_deadline_missed(self.deadline_missed)
            .with_members(self.members.clone())
            .build()
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        if let Some(peer_id) = self.non_member_peer(&command) {
            return (
                vec![Outcome::NonMemberIgnored { peer_id }],
                Box::new(Self {
                    members: self.members.clone(),
                    pinging_peers: self.pinging_peers.clone(),
                    collecting_peers: self.collecting_peers.clone(),
                    deadline_missed: self.deadline_missed,
                }),
            );
        }

        match command {
            Command::ParticipationObserved { peer_id } => {
                let step = PingingStep::new(self.pinging_peers.clone(), peer_id);

                (
                    step.outcomes().to_vec(),
                    Box::new(Self {
                        members: self.members.clone(),
                        pinging_peers: step.confirmed_peers().to_vec(),
                        collecting_peers: self.collecting_peers.clone(),
                        deadline_missed: self.deadline_missed,
                    }),
                )
            }

            Command::ReadyObserved { peer_id } => {
                let is_dup = self.collecting_peers.contains(&peer_id);
                let mut new_collecting = self.collecting_peers.clone();
                if !is_dup {
                    new_collecting.push(peer_id);
                }

                let outcome = if is_dup {
                    Outcome::DuplicateReadyIgnored { peer_id }
                } else {
                    Outcome::ReadyAccepted { peer_id }
                };

                (
                    vec![outcome],
                    Box::new(Self {
                        members: self.members.clone(),
                        pinging_peers: self.pinging_peers.clone(),
                        collecting_peers: new_collecting,
                        deadline_missed: self.deadline_missed,
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
                vec![Outcome::DeadlineMissed {
                    confirmed_count: self.collecting_peers.len(),
                }],
                Box::new(Self {
                    members: self.members.clone(),
                    pinging_peers: self.pinging_peers.clone(),
                    collecting_peers: self.collecting_peers.clone(),
                    deadline_missed: true,
                }),
            ),

            Command::JoinRequested { .. }
            | Command::JoinApproved { .. }
            | Command::JoinRejected { .. } => {
                let join = JoinStep::new(self.members.clone(), &command);
                (
                    join.outcomes().to_vec(),
                    Box::new(Self {
                        members: join.members().clone(),
                        pinging_peers: self.pinging_peers.clone(),
                        collecting_peers: self.collecting_peers.clone(),
                        deadline_missed: self.deadline_missed,
                    }),
                )
            }

            Command::Probe => unreachable!("Probe handled in Faction::process"),
        }
    }
}
