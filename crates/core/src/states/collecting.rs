// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::conclusion::Conclusion;
use crate::config::Config;
use crate::members::Members;
use crate::outcome::Outcome;
use crate::peer_state::PeerState;
use crate::quorum_policy::QuorumPolicy;
use crate::reconfiguration::Reconfiguration;
use crate::stage::Stage;
use crate::state::State;
use crate::types::PeerId;

use super::bootstrapped::Bootstrapped;
use super::membership_decision::MembershipDecision;
use super::membership_step::MembershipStep;
use super::ready_step::ReadyStep;

pub struct Collecting {
    members: Members,
    quorum_policy: QuorumPolicy,
    collecting_peers: Vec<PeerId>,
    pinged_peers: Vec<PeerId>,
    deadline_missed: bool,
}

impl Collecting {
    #[must_use]
    pub fn new(
        members: Members,
        quorum_policy: QuorumPolicy,
        collecting_peers: Vec<PeerId>,
        pinged_peers: Vec<PeerId>,
        deadline_missed: bool,
    ) -> Self {
        Self {
            members,
            quorum_policy,
            collecting_peers,
            pinged_peers,
            deadline_missed,
        }
    }

    #[must_use]
    pub fn resolve(self, is_quorum: bool) -> Stage {
        if is_quorum {
            Stage::Bootstrapped(Bootstrapped::new(
                self.members,
                self.quorum_policy,
                self.pinged_peers,
                self.collecting_peers,
            ))
        } else {
            Stage::Collecting(self)
        }
    }
}

impl State for Collecting {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Collecting)
            .with_is_pinging_completed(true)
            .with_pinging_peers(self.pinged_peers.clone())
            .with_collecting_peers(self.collecting_peers.clone())
            .with_deadline_missed(self.deadline_missed)
            .with_required_count(self.quorum_policy.threshold())
            .with_members(self.members.clone())
            .with_reconfiguration(Reconfiguration::None)
            .build()
    }

    fn step(&self, command: Command, _config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        if let Some(peer_id) = self.members.non_member_observed(&command) {
            Some((
                vec![Outcome::NonMemberIgnored { peer_id }],
                Stage::Collecting(Self::new(
                    self.members.clone(),
                    self.quorum_policy,
                    self.collecting_peers.clone(),
                    self.pinged_peers.clone(),
                    self.deadline_missed,
                )),
            ))
        } else if let Some(decision) = MembershipDecision::of(&command) {
            let membership =
                MembershipStep::new(self.members.clone(), self.quorum_policy, decision);
            let collecting_peers = membership.members().retained(&self.collecting_peers);
            let is_quorum = membership
                .quorum_policy()
                .is_satisfied(collecting_peers.len());
            let conclusion = is_quorum.then_some(Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            });
            Some((
                membership
                    .outcomes()
                    .iter()
                    .cloned()
                    .chain(conclusion)
                    .collect(),
                Self::new(
                    membership.members().clone(),
                    membership.quorum_policy(),
                    collecting_peers,
                    membership.members().retained(&self.pinged_peers),
                    self.deadline_missed,
                )
                .resolve(is_quorum),
            ))
        } else {
            match command {
                Command::ParticipationObserved { peer_id } => Some((
                    vec![Outcome::AcknowledgeRejoin { peer_id }],
                    Stage::Collecting(Self::new(
                        self.members.clone(),
                        self.quorum_policy,
                        self.collecting_peers.clone(),
                        self.pinged_peers.clone(),
                        self.deadline_missed,
                    )),
                )),

                Command::ReadyObserved { peer_id } => {
                    let step =
                        ReadyStep::new(self.collecting_peers.clone(), peer_id, self.quorum_policy);
                    Some((
                        step.outcomes().to_vec(),
                        Self::new(
                            self.members.clone(),
                            self.quorum_policy,
                            step.confirmed_peers().to_vec(),
                            self.pinged_peers.clone(),
                            self.deadline_missed,
                        )
                        .resolve(step.is_quorum()),
                    ))
                }

                Command::DeadlineExpired => Some((
                    vec![Outcome::DeadlineMissed {
                        confirmed_count: self.collecting_peers.len(),
                    }],
                    Stage::Collecting(Self::new(
                        self.members.clone(),
                        self.quorum_policy,
                        self.collecting_peers.clone(),
                        self.pinged_peers.clone(),
                        true,
                    )),
                )),

                _ => None,
            }
        }
    }
}
