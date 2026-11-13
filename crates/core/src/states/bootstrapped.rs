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
use crate::quorum::Quorum;
use crate::quorum_policy::QuorumPolicy;
use crate::reconfiguration::Reconfiguration;
use crate::stage::Stage;
use crate::state::State;
use crate::states::liveness::Liveness;
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
    liveness: Liveness,
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
            liveness: Liveness::default(),
        }
    }

    // What a concluded node answers whatever its addition: a peer still trying
    // to bootstrap, a membership decision -- and the quorum it crosses -- and
    // its members' liveness. Anything else it does not admit.
    #[must_use]
    pub fn answer(&self, command: Command, config: &Config) -> Option<(Vec<Outcome>, Self)> {
        if let Some(decision) = MembershipDecision::of(&command) {
            let membership =
                MembershipStep::new(self.members.clone(), self.quorum_policy, decision);
            let crossing = Liveness::crossing(
                self.liveness
                    .holds_quorum(&self.members, self.quorum_policy),
                self.liveness
                    .holds_quorum(membership.members(), membership.quorum_policy()),
            );
            Some((
                membership
                    .outcomes()
                    .iter()
                    .cloned()
                    .chain(crossing)
                    .collect(),
                Self {
                    members: membership.members().clone(),
                    quorum_policy: membership.quorum_policy(),
                    pinged_peers: membership.members().retained(&self.pinged_peers),
                    collected_peers: membership.members().retained(&self.collected_peers),
                    liveness: self.liveness.of_members(membership.members()),
                },
            ))
        } else if let Some((outcomes, liveness)) =
            self.liveness
                .step(command, &self.members, self.quorum_policy, config)
        {
            Some((
                outcomes,
                Self {
                    liveness,
                    ..self.clone()
                },
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

impl Bootstrapped {
    // Whether the quorum can spare the member: a suspect it always can; any
    // other, if the members not suspected still number the count without it.
    fn can_spare(&self, peer_id: PeerId) -> bool {
        self.liveness.suspected().contains(&peer_id)
            || self
                .members
                .without(peer_id)
                .is_some_and(|rest| self.liveness.holds_quorum(&rest, self.quorum_policy))
    }
}

impl State for Bootstrapped {
    fn step(&self, command: Command, config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        match command {
            Command::AdditionRequested { peer_id } if self.members.is_member(peer_id) => Some((
                vec![Outcome::DuplicateMemberIgnored { peer_id }],
                Stage::Bootstrapped(self.clone()),
            )),
            Command::AdditionRequested { peer_id } => Some((
                vec![Outcome::SendMembership { peer_id }],
                Stage::Reconfiguring(Reconfiguring::new(
                    self.clone(),
                    Reconfiguration::AdditionRequested { peer_id },
                )),
            )),
            Command::RemovalRequested { peer_id } if !self.members.is_member(peer_id) => Some((
                vec![Outcome::NonMemberIgnored { peer_id }],
                Stage::Bootstrapped(self.clone()),
            )),
            Command::RemovalRequested { peer_id } if !self.can_spare(peer_id) => Some((
                vec![Outcome::RemovalRefused { peer_id }],
                Stage::Bootstrapped(self.clone()),
            )),
            Command::RemovalRequested { peer_id } => Some((
                vec![Outcome::ProposeRemoval { peer_id }],
                Stage::Reconfiguring(Reconfiguring::new(
                    self.clone(),
                    Reconfiguration::RemovalRequested { peer_id },
                )),
            )),
            _ => self
                .answer(command, config)
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
            .with_reconfiguration(Reconfiguration::None)
            .with_suspected(self.liveness.suspected().to_vec())
            .with_misses(self.liveness.misses().to_vec())
            .with_last_probed(self.liveness.last_probed())
            .with_quorum(
                if self
                    .liveness
                    .holds_quorum(&self.members, self.quorum_policy)
                {
                    Quorum::Held
                } else {
                    Quorum::Lost
                },
            )
            .build()
    }
}
