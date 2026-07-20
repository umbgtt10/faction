// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::state::State;
use crate::states::join_step::JoinStep;
use crate::types::PeerId;

pub struct Bootstrapped {
    members: Members,
    pinged_peers: Vec<PeerId>,
    collected_peers: Vec<PeerId>,
}

impl Bootstrapped {
    #[must_use]
    pub fn new(members: Members, pinged_peers: Vec<PeerId>, collected_peers: Vec<PeerId>) -> Self {
        Self {
            members,
            pinged_peers,
            collected_peers,
        }
    }
}

impl State for Bootstrapped {
    fn step(&self, command: Command, _config: &Config) -> (Vec<Outcome>, Box<dyn State>) {
        match command {
            Command::ParticipationObserved { peer_id } => {
                let outcome = if self.members.is_member(peer_id) {
                    Outcome::AcknowledgeRejoin { peer_id }
                } else {
                    Outcome::NonMemberIgnored { peer_id }
                };
                (
                    vec![outcome],
                    Box::new(Self::new(
                        self.members.clone(),
                        self.pinged_peers.clone(),
                        self.collected_peers.clone(),
                    )),
                )
            }
            Command::JoinRequested { .. }
            | Command::JoinApproved { .. }
            | Command::JoinRejected { .. } => {
                let join = JoinStep::new(self.members.clone(), &command);
                (
                    join.outcomes().to_vec(),
                    Box::new(Self::new(
                        join.members().clone(),
                        self.pinged_peers.clone(),
                        self.collected_peers.clone(),
                    )),
                )
            }
            _ => unreachable!("accept() rejects this command for Bootstrapped"),
        }
    }

    fn accept(&self, command: &Command) -> bool {
        matches!(
            command,
            Command::ParticipationObserved { .. }
                | Command::JoinRequested { .. }
                | Command::JoinApproved { .. }
                | Command::JoinRejected { .. }
        )
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        previous
            .clone()
            .with_peer_state(PeerState::Bootstrapped)
            .with_is_pinging_completed(true)
            .with_pinging_peers(self.pinged_peers.clone())
            .with_collecting_peers(self.collected_peers.clone())
    }

    fn admissible_commands(&self) -> Vec<Command> {
        vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::JoinRequested { peer_id: 0 },
            Command::JoinApproved { peer_id: 0 },
            Command::JoinRejected { peer_id: 0 },
            Command::Probe,
        ]
    }
}
