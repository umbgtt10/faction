// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::addition::Addition;
use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::quorum_policy::QuorumPolicy;
use crate::stage::Stage;
use crate::state::State;
use crate::states::membership_decision::MembershipDecision;
use crate::states::membership_step::MembershipStep;
use crate::states::reconfiguring::Reconfiguring;
use crate::types::PeerId;

#[derive(Debug, Clone)]
pub struct Bootstrapped {
    members: Members,
    quorum_policy: QuorumPolicy,
    pinged_peers: Vec<PeerId>,
    collected_peers: Vec<PeerId>,
}

impl Bootstrapped {
    #[must_use]
    pub fn new(
        members: Members,
        quorum_policy: QuorumPolicy,
        pinged_peers: Vec<PeerId>,
        collected_peers: Vec<PeerId>,
    ) -> Self {
        Self {
            members,
            quorum_policy,
            pinged_peers,
            collected_peers,
        }
    }

    // What a concluded node answers whatever its addition: a peer still trying
    // to bootstrap, and a membership decision. Anything else it does not admit.
    #[must_use]
    pub fn answer(&self, command: Command) -> Option<(Vec<Outcome>, Self)> {
        if let Some(decision) = MembershipDecision::of(&command) {
            let membership =
                MembershipStep::new(self.members.clone(), self.quorum_policy, decision);
            Some((
                membership.outcomes().to_vec(),
                Self::new(
                    membership.members().clone(),
                    membership.quorum_policy(),
                    self.pinged_peers.clone(),
                    self.collected_peers.clone(),
                ),
            ))
        } else {
            match command {
                Command::ParticipationObserved { peer_id } => {
                    let outcome = if self.members.is_member(peer_id) {
                        Outcome::AcknowledgeRejoin { peer_id }
                    } else {
                        Outcome::NonMemberIgnored { peer_id }
                    };
                    Some((vec![outcome], self.clone()))
                }
                _ => None,
            }
        }
    }
}

impl State for Bootstrapped {
    fn step(&self, command: Command, _config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        match command {
            Command::AdditionRequested { peer_id } if self.members.is_member(peer_id) => Some((
                vec![Outcome::DuplicateMemberIgnored { peer_id }],
                Stage::Bootstrapped(self.clone()),
            )),
            Command::AdditionRequested { peer_id } => Some((
                vec![Outcome::SendMembership { peer_id }],
                Stage::Reconfiguring(Reconfiguring::new(self.clone(), peer_id)),
            )),
            _ => self
                .answer(command)
                .map(|(outcomes, concluded)| (outcomes, Stage::Bootstrapped(concluded))),
        }
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Bootstrapped)
            .with_is_pinging_completed(true)
            .with_pinging_peers(self.pinged_peers.clone())
            .with_collecting_peers(self.collected_peers.clone())
            .with_required_count(self.quorum_policy.threshold())
            .with_members(self.members.clone())
            .with_addition(Addition::None)
            .build()
    }
}
