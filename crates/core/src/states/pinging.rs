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
use crate::types::PeerId;

use super::collecting::Collecting;
use super::local_completion_step::LocalCompletionStep;
use super::membership_decision::MembershipDecision;
use super::membership_step::MembershipStep;
use super::pinging_step::PingingStep;

#[derive(Debug, Clone)]
pub struct Pinging {
    members: Members,
    quorum_policy: QuorumPolicy,
    pinging_peers: Vec<PeerId>,
    collecting_peers: Vec<PeerId>,
    deadline_missed: bool,
}

impl Pinging {
    #[must_use]
    pub fn new(members: Members, quorum_policy: QuorumPolicy) -> Self {
        Self {
            members,
            quorum_policy,
            pinging_peers: Vec::new(),
            collecting_peers: Vec::new(),
            deadline_missed: false,
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
            .with_required_count(self.quorum_policy.threshold())
            .with_members(self.members.clone())
            .with_addition(Addition::None)
            .build()
    }

    fn step(&self, command: Command, config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        if let Some(peer_id) = self.members.non_member_observed(&command) {
            Some((
                vec![Outcome::NonMemberIgnored { peer_id }],
                Stage::Pinging(self.clone()),
            ))
        } else if let Some(decision) = MembershipDecision::of(&command) {
            let membership =
                MembershipStep::new(self.members.clone(), self.quorum_policy, decision);
            Some((
                membership.outcomes().to_vec(),
                Stage::Pinging(Self {
                    members: membership.members().clone(),
                    quorum_policy: membership.quorum_policy(),
                    ..self.clone()
                }),
            ))
        } else {
            match command {
                Command::ParticipationObserved { peer_id } => {
                    let step = PingingStep::new(self.pinging_peers.clone(), peer_id);

                    Some((
                        step.outcomes().to_vec(),
                        Stage::Pinging(Self {
                            pinging_peers: step.confirmed_peers().to_vec(),
                            ..self.clone()
                        }),
                    ))
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

                    Some((
                        vec![outcome],
                        Stage::Pinging(Self {
                            collecting_peers: new_collecting,
                            ..self.clone()
                        }),
                    ))
                }

                Command::LocalParticipationCompleted => {
                    let step = LocalCompletionStep::new(
                        self.collecting_peers.clone(),
                        config.peer_id(),
                        self.quorum_policy,
                    );

                    Some((
                        step.outcomes().to_vec(),
                        Collecting::new(
                            self.members.clone(),
                            self.quorum_policy,
                            step.confirmed_peers().to_vec(),
                            self.pinging_peers.clone(),
                            self.deadline_missed,
                        )
                        .resolve(step.is_quorum()),
                    ))
                }

                Command::DeadlineExpired => Some((
                    vec![Outcome::DeadlineMissed {
                        confirmed_count: self.collecting_peers.len(),
                    }],
                    Stage::Pinging(Self {
                        deadline_missed: true,
                        ..self.clone()
                    }),
                )),

                _ => None,
            }
        }
    }
}
