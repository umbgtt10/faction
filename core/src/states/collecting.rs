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
use super::join_step::JoinStep;
use super::ready_step::ReadyStep;

pub struct Collecting {
    members: Members,
    collecting_peers: Vec<PeerId>,
    pinged_peers: Vec<PeerId>,
    deadline_missed: bool,
}

impl Collecting {
    #[must_use]
    pub fn new(
        members: Members,
        collecting_peers: Vec<PeerId>,
        pinged_peers: Vec<PeerId>,
        deadline_missed: bool,
    ) -> Self {
        Self {
            members,
            collecting_peers,
            pinged_peers,
            deadline_missed,
        }
    }

    fn compute_new_state(&self, is_quorum: bool, confirmed_peers: Vec<PeerId>) -> Box<dyn State> {
        if is_quorum {
            Box::new(Bootstrapped::new(
                self.members.clone(),
                self.pinged_peers.clone(),
                confirmed_peers,
            ))
        } else {
            Box::new(Self::new(
                self.members.clone(),
                confirmed_peers,
                self.pinged_peers.clone(),
                self.deadline_missed,
            ))
        }
    }

    fn non_member_peer(&self, command: &Command) -> Option<PeerId> {
        match command {
            Command::ReadyObserved { peer_id, .. } if !self.members.is_member(*peer_id) => {
                Some(*peer_id)
            }
            _ => None,
        }
    }
}

impl State for Collecting {
    fn accept(&self, command: &Command) -> bool {
        matches!(
            command,
            Command::ReadyObserved { .. }
                | Command::DeadlineExpired
                | Command::JoinRequested { .. }
                | Command::JoinApproved { .. }
                | Command::JoinRejected { .. }
        )
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![
            Command::ReadyObserved { peer_id: 0 },
            Command::DeadlineExpired,
            Command::JoinRequested { peer_id: 0 },
            Command::JoinApproved { peer_id: 0 },
            Command::JoinRejected { peer_id: 0 },
            Command::Probe,
        ]
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Collecting)
            .with_is_pinging_completed(true)
            .with_pinging_peers(self.pinged_peers.clone())
            .with_collecting_peers(self.collecting_peers.clone())
            .with_deadline_missed(self.deadline_missed)
            .with_members(self.members.clone())
            .build()
    }

    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        if let Some(peer_id) = self.non_member_peer(&command) {
            return (
                vec![Outcome::NonMemberIgnored { peer_id }],
                Box::new(Self::new(
                    self.members.clone(),
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
                    self.members.clone(),
                    self.collecting_peers.clone(),
                    self.pinged_peers.clone(),
                    true,
                )),
            ),

            Command::JoinRequested { .. }
            | Command::JoinApproved { .. }
            | Command::JoinRejected { .. } => {
                let join = JoinStep::new(self.members.clone(), &command);
                (
                    join.outcomes().to_vec(),
                    Box::new(Self::new(
                        join.members().clone(),
                        self.collecting_peers.clone(),
                        self.pinged_peers.clone(),
                        self.deadline_missed,
                    )),
                )
            }

            Command::Probe => {
                unreachable!("Probe handled in Faction::process")
            }
        }
    }
}
