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
use crate::quorum_policy::QuorumPolicy;
use crate::stage::Stage;
use crate::state::State;
use crate::types::Epoch;

use super::collecting::Collecting;
use super::epoch_rule::EpochRule;
use super::local_completion_step::LocalCompletionStep;
use super::membership_decision::MembershipDecision;
use super::membership_step::MembershipStep;
use super::pinging_step::PingingStep;
use super::ready_before_completion_step::ReadyBeforeCompletionStep;
use super::removed::Removed;

#[derive(Debug, Clone)]
pub struct Pinging<P: PeerId = u64> {
    members: Members<P>,
    quorum_policy: QuorumPolicy,
    pinging_peers: Vec<P>,
    collecting_peers: Vec<P>,
    deadline_missed: bool,
}

impl<P: PeerId> Pinging<P> {
    #[must_use]
    pub fn new(members: Members<P>, quorum_policy: QuorumPolicy) -> Self {
        Self {
            members,
            quorum_policy,
            pinging_peers: Vec::new(),
            collecting_peers: Vec::new(),
            deadline_missed: false,
        }
    }
}

impl<P: PeerId> State<P> for Pinging<P> {
    fn cluster_view(&self, previous: &ClusterView<P>) -> ClusterView<P> {
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Pinging)
            .with_pinging_peers(self.pinging_peers.clone())
            .with_collecting_peers(self.collecting_peers.clone())
            .with_deadline_missed(self.deadline_missed)
            .with_required_count(self.quorum_policy.threshold())
            .with_members(self.members.clone())
            .with_reconfigurations(Vec::new())
            .build()
    }

    fn step(
        &self,
        command: Command<P>,
        config: &Config<P>,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome<P>>, Stage<P>)> {
        if let Some(departure) = Removed::departure(
            &command,
            config.peer_id(),
            &self.members,
            self.quorum_policy,
        ) {
            Some(departure)
        } else if let Some(uncounted) = EpochRule::new(epoch).uncounted(&command, &self.members) {
            Some((uncounted, Stage::Pinging(self.clone())))
        } else if let Some(decision) = MembershipDecision::of(&command) {
            let membership =
                MembershipStep::new(self.members.clone(), self.quorum_policy, decision);
            Some((
                membership.outcomes().to_vec(),
                Stage::Pinging(Self {
                    members: membership.members().clone(),
                    quorum_policy: membership.quorum_policy(),
                    pinging_peers: membership.members().retained(&self.pinging_peers),
                    collecting_peers: membership.members().retained(&self.collecting_peers),
                    ..self.clone()
                }),
            ))
        } else {
            match command {
                Command::ParticipationObserved { peer_id, .. } => {
                    let step = PingingStep::new(self.pinging_peers.clone(), peer_id);

                    Some((
                        step.outcomes().to_vec(),
                        Stage::Pinging(Self {
                            pinging_peers: step.confirmed_peers().to_vec(),
                            ..self.clone()
                        }),
                    ))
                }

                Command::ReadyObserved { peer_id, .. } => {
                    let step =
                        ReadyBeforeCompletionStep::new(self.collecting_peers.clone(), peer_id);

                    Some((
                        step.outcomes().to_vec(),
                        Stage::Pinging(Self {
                            collecting_peers: step.collecting_peers().to_vec(),
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
