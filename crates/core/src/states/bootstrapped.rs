// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::instance::peer_id::PeerId;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::quorum::Quorum;
use crate::quorum_policy::QuorumPolicy;
use crate::reconfiguration::Reconfiguration;
use crate::stage::Stage;
use crate::state::State;
use crate::states::epoch_rule::EpochRule;
use crate::states::in_flight::InFlight;
use crate::states::liveness::Liveness;
use crate::states::membership_decision::MembershipDecision;
use crate::states::membership_step::MembershipStep;
use crate::states::probing::Probing;
use crate::states::removed::Removed;
use crate::states::suspicion::Suspicion;
use crate::types::Epoch;

#[derive(Debug, Clone)]
pub struct Bootstrapped<P: PeerId = u64> {
    members: Members<P>,
    quorum_policy: QuorumPolicy,
    pinged_peers: Vec<P>,
    collected_peers: Vec<P>,
    liveness: Liveness<P>,
}

impl<P: PeerId> Bootstrapped<P> {
    #[must_use]
    pub fn new(
        members: Members<P>,
        quorum_policy: QuorumPolicy,
        pinged_peers: Vec<P>,
        collected_peers: Vec<P>,
    ) -> Self {
        Self {
            members,
            quorum_policy,
            pinged_peers,
            collected_peers,
            liveness: Liveness::new(Probing::default(), Suspicion::default()),
        }
    }

    // What a concluded node answers whatever its addition: a peer still trying
    // to bootstrap -- refused from behind -- a membership decision -- and the
    // quorum it crosses -- and its members' liveness. Anything else it does not
    // admit.
    #[must_use]
    pub fn answer(
        &self,
        command: Command<P>,
        config: &Config<P>,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome<P>>, Self)> {
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

impl<P: PeerId> Bootstrapped<P> {
    // Whether the quorum can spare the member: a suspect it always can; any
    // other, if the members not suspected still number the count without it
    // and every other member whose removal, `removing`, is in flight (§12).
    #[must_use]
    pub fn can_spare(&self, peer_id: P, removing: &[P]) -> bool {
        self.liveness.suspected().contains(&peer_id)
            || removing
                .iter()
                .filter(|removed| **removed != peer_id)
                .fold(self.members.without(peer_id), |rest, removed| {
                    rest.and_then(|rest| rest.without(*removed))
                })
                .is_some_and(|rest| self.liveness.holds_quorum(&rest, self.quorum_policy))
    }

    // A change of membership applied: `outcomes`, then the quorum it crosses;
    // the peers gathered and the liveness kept to the new members.
    #[must_use]
    pub fn changed(
        &self,
        outcomes: Vec<Outcome<P>>,
        members: &Members<P>,
        quorum_policy: QuorumPolicy,
    ) -> (Vec<Outcome<P>>, Self) {
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
    pub fn departure(
        &self,
        command: &Command<P>,
        peer_id: P,
    ) -> Option<(Vec<Outcome<P>>, Stage<P>)> {
        Removed::departure(command, peer_id, &self.members, self.quorum_policy)
    }

    // A change requested of the node with `in_flight` already (§11, §12): a
    // peer with a change in flight ignored as a duplicate; one already a
    // member, or no member to remove, ignored; a removal the quorum cannot
    // spare, the removals in flight counted, refused; otherwise the change
    // requested last, the node in the stage the changes then make. Any other
    // command is no request.
    #[must_use]
    pub fn requested(
        &self,
        command: &Command<P>,
        in_flight: &InFlight<P>,
        config: &Config<P>,
    ) -> Option<(Vec<Outcome<P>>, Stage<P>)> {
        let (outcome, change) = match *command {
            Command::AdditionRequested { peer_id } | Command::RemovalRequested { peer_id }
                if in_flight.named(peer_id).is_some() =>
            {
                (Outcome::DuplicateChangeIgnored { peer_id }, None)
            }
            Command::AdditionRequested { peer_id } if self.members.is_member(peer_id) => {
                (Outcome::DuplicateMemberIgnored { peer_id }, None)
            }
            Command::AdditionRequested { peer_id } => (
                Outcome::SendMembership { peer_id },
                Some(Reconfiguration::AdditionRequested { peer_id }),
            ),
            Command::RemovalRequested { peer_id } if !self.members.is_member(peer_id) => {
                (Outcome::NonMemberIgnored { peer_id }, None)
            }
            Command::RemovalRequested { peer_id }
                if !self.can_spare(peer_id, &in_flight.removals()) =>
            {
                (Outcome::RemovalRefused { peer_id }, None)
            }
            Command::RemovalRequested { peer_id } => (
                Outcome::ProposeRemoval { peer_id },
                Some(Reconfiguration::RemovalRequested { peer_id }),
            ),
            _ => return None,
        };
        let changes = change.map_or_else(|| in_flight.clone(), |change| in_flight.with(change));
        Some((vec![outcome], changes.stage(self.clone(), config)))
    }
}

impl<P: PeerId> State<P> for Bootstrapped<P> {
    fn step(
        &self,
        command: Command<P>,
        config: &Config<P>,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome<P>>, Stage<P>)> {
        if let Some(departure) = self.departure(&command, config.peer_id()) {
            return Some(departure);
        }
        self.requested(&command, &InFlight::default(), config)
            .or_else(|| {
                self.answer(command, config, epoch)
                    .map(|(outcomes, concluded)| (outcomes, Stage::Bootstrapped(concluded)))
            })
    }

    fn cluster_view(&self, previous: &ClusterView<P>) -> ClusterView<P> {
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Bootstrapped)
            .with_is_pinging_completed(true)
            .with_pinging_peers(self.pinged_peers.clone())
            .with_collecting_peers(self.collected_peers.clone())
            .with_required_count(self.quorum_policy.threshold())
            .with_members(self.members.clone())
            .with_reconfigurations(Vec::new())
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
