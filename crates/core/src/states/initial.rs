// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

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
use crate::stage::Stage;
use crate::state::State;
use crate::states::membership_decision::MembershipDecision;
use crate::states::membership_step::MembershipStep;
use crate::states::pinging::Pinging;
use crate::states::removed::Removed;
use crate::types::Epoch;

#[derive(Debug, Clone)]
pub struct Initial {
    members: Members,
    quorum_policy: QuorumPolicy,
}

impl Initial {
    #[must_use]
    pub fn new(members: Members, quorum_policy: QuorumPolicy) -> Self {
        Self {
            members,
            quorum_policy,
        }
    }
}

impl State for Initial {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Fresh)
            .with_is_pinging_completed(false)
            .with_pinging_peers(Vec::new())
            .with_collecting_peers(Vec::new())
            .with_required_count(self.quorum_policy.threshold())
            .with_deadline_missed(false)
            .with_members(self.members.clone())
            .with_reconfigurations(Vec::new())
            .with_suspected(Vec::new())
            .with_misses(Vec::new())
            .with_last_probed(None)
            .with_quorum(Quorum::NotTracked)
            .build()
    }

    fn step(
        &self,
        command: Command,
        config: &Config,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome>, Stage)> {
        if let Some(departure) = Removed::departure(
            &command,
            config.peer_id(),
            &self.members,
            self.quorum_policy,
        ) {
            Some(departure)
        } else if let Some(decision) = MembershipDecision::of(&command) {
            let membership =
                MembershipStep::new(self.members.clone(), self.quorum_policy, decision);
            Some((
                membership.outcomes().to_vec(),
                Stage::Initial(Self::new(
                    membership.members().clone(),
                    membership.quorum_policy(),
                )),
            ))
        } else {
            match command {
                Command::DeadlineExpired => None,
                _ => Pinging::new(self.members.clone(), self.quorum_policy)
                    .step(command, config, epoch),
            }
        }
    }
}
