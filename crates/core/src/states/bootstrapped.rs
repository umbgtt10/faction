// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::stage::Stage;
use crate::state::State;
use crate::states::join_decision::JoinDecision;
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
    fn step(&self, command: Command, _config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        if let Some(decision) = JoinDecision::of(&command) {
            let join = JoinStep::new(self.members.clone(), decision);
            Some((
                join.outcomes().to_vec(),
                Stage::Bootstrapped(Self::new(
                    join.members().clone(),
                    self.pinged_peers.clone(),
                    self.collected_peers.clone(),
                )),
            ))
        } else {
            match command {
                Command::ParticipationObserved { peer_id } => {
                    let outcome = if self.members.is_member(peer_id) {
                        Outcome::AcknowledgeRejoin { peer_id }
                    } else {
                        Outcome::NonMemberIgnored { peer_id }
                    };
                    Some((
                        vec![outcome],
                        Stage::Bootstrapped(Self::new(
                            self.members.clone(),
                            self.pinged_peers.clone(),
                            self.collected_peers.clone(),
                        )),
                    ))
                }
                _ => None,
            }
        }
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Bootstrapped)
            .with_is_pinging_completed(true)
            .with_pinging_peers(self.pinged_peers.clone())
            .with_collecting_peers(self.collected_peers.clone())
            .with_members(self.members.clone())
            .build()
    }
}
