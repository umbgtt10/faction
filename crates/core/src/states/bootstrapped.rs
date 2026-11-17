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
use crate::states::epoch_rule::EpochRule;
use crate::states::liveness::Liveness;
use crate::states::membership_decision::MembershipDecision;
use crate::states::membership_step::MembershipStep;
use crate::states::reconfiguring::Reconfiguring;
use crate::states::removed::Removed;
use crate::types::Epoch;
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
    // to bootstrap -- refused from behind -- a membership decision -- and the
    // quorum it crosses -- and its members' liveness. Anything else it does not
    // admit.
    #[must_use]
    pub fn answer(
        &self,
        command: Command,
        config: &Config,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome>, Self)> {
        if let Some(decision) = MembershipDecision::of(&command) {
            let membership =
                MembershipStep::new(self.members.clone(), self.quorum_policy, decision);
            Some(self.changed(
                membership.outcomes().to_vec(),
                membership.members(),
                membership.quorum_policy(),
            ))
        } else if let Some((outcomes, liveness)) =
            self.liveness
                .step(&command, &self.members, self.quorum_policy, config)
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
                Command::ParticipationObserved { peer_id, .. }
                    if !self.members.is_member(peer_id) =>
                {
                    Some((vec![Outcome::NonMemberIgnored { peer_id }], self.clone()))
                }
                Command::ParticipationObserved {
                    peer_id,
                    epoch: claimed,
                } => Some((
                    EpochRule::new(epoch)
                        .refused(peer_id, claimed)
                        .unwrap_or_else(|| vec![Outcome::AcknowledgeRejoin { peer_id }]),
                    self.clone(),
                )),
                _ => None,
            }
        }
    }
}

impl Bootstrapped {
    // Whether the quorum can spare the member: a suspect it always can; any
    // other, if the members not suspected still number the count without it.
    #[must_use]
    pub fn can_spare(&self, peer_id: PeerId) -> bool {
        self.liveness.suspected().contains(&peer_id)
            || self
                .members
                .without(peer_id)
                .is_some_and(|rest| self.liveness.holds_quorum(&rest, self.quorum_policy))
    }

    // A change of membership applied: `outcomes`, then the quorum it crosses;
    // the peers gathered and the liveness kept to the new members.
    #[must_use]
    pub fn changed(
        &self,
        outcomes: Vec<Outcome>,
        members: &Members,
        quorum_policy: QuorumPolicy,
    ) -> (Vec<Outcome>, Self) {
        let crossing = Liveness::crossing(
            self.liveness
                .holds_quorum(&self.members, self.quorum_policy),
            self.liveness.holds_quorum(members, quorum_policy),
        );
        (
            outcomes.into_iter().chain(crossing).collect(),
            Self {
                members: members.clone(),
                quorum_policy,
                pinged_peers: members.retained(&self.pinged_peers),
                collected_peers: members.retained(&self.collected_peers),
                liveness: self.liveness.of_members(members),
            },
        )
    }

    // What removes the node `peer_id`: its own removal committed, or a
    // membership that does not name it.
    #[must_use]
    pub fn departure(&self, command: &Command, peer_id: PeerId) -> Option<(Vec<Outcome>, Stage)> {
        Removed::departure(command, peer_id, &self.members, self.quorum_policy)
    }
}

impl State for Bootstrapped {
    fn step(
        &self,
        command: Command,
        config: &Config,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome>, Stage)> {
        if let Some(departure) = self.departure(&command, config.peer_id()) {
            return Some(departure);
        }
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
                .answer(command, config, epoch)
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
