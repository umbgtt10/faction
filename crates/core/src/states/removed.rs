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
use crate::stage::Stage;
use crate::state::State;
use crate::states::adoption::Adoption;
use crate::types::Epoch;

// A node removed from its own cluster: it admits a membership and a probe
// alone, and shows the membership it holds but itself, the peers it gathered
// among its members, and no liveness. A membership that names it takes it back
// to `Initial`.
#[derive(Debug, Clone)]
pub struct Removed<P: PeerId = u64> {
    peer_id: P,
    members: Members<P>,
    quorum_policy: QuorumPolicy,
}

impl<P: PeerId> Removed<P> {
    #[must_use]
    pub fn new(peer_id: P, members: Members<P>, quorum_policy: QuorumPolicy) -> Self {
        Self {
            peer_id,
            members,
            quorum_policy,
        }
    }

    // What removes the node `peer_id`, in every stage that admits it, holding
    // `members` and their count: its own removal committed, or a membership
    // that does not name it (§12, §16).
    #[must_use]
    pub fn departure(
        command: &Command<P>,
        peer_id: P,
        members: &Members<P>,
        quorum_policy: QuorumPolicy,
    ) -> Option<(Vec<Outcome<P>>, Stage<P>)> {
        match *command {
            Command::RemovalCommitted {
                peer_id: removed, ..
            } if removed == peer_id => Some(Self::own_removal(peer_id, members, quorum_policy)),
            _ => Adoption::of(command).and_then(|adoption| adoption.removal(peer_id)),
        }
    }

    // What a node answers its own removal, committed: the node removed, and
    // `Removed`, holding the members and the count it held.
    #[must_use]
    pub fn own_removal(
        peer_id: P,
        members: &Members<P>,
        quorum_policy: QuorumPolicy,
    ) -> (Vec<Outcome<P>>, Stage<P>) {
        (
            vec![Outcome::MemberRemoved { peer_id }],
            Stage::Removed(Self::new(peer_id, members.clone(), quorum_policy)),
        )
    }
}

impl<P: PeerId> State<P> for Removed<P> {
    fn step(
        &self,
        command: Command<P>,
        _config: &Config<P>,
        _epoch: Epoch,
    ) -> Option<(Vec<Outcome<P>>, Stage<P>)> {
        Adoption::of(&command).map(|adoption| adoption.taken_back(self.peer_id))
    }

    fn cluster_view(&self, previous: &ClusterView<P>) -> ClusterView<P> {
        let members = Members::new(
            self.members
                .as_slice()
                .iter()
                .copied()
                .filter(|member| *member != self.peer_id)
                .collect(),
        );
        ClusterViewBuilder::from_view(previous)
            .with_peer_state(PeerState::Removed)
            .with_pinging_peers(members.retained(previous.pinging_peers()))
            .with_collecting_peers(members.retained(previous.collecting_peers()))
            .with_required_count(self.quorum_policy.threshold())
            .with_members(members)
            .with_reconfigurations(Vec::new())
            .with_suspected(Vec::new())
            .with_misses(Vec::new())
            .with_last_probed(None)
            .with_quorum(Quorum::NotTracked)
            .build()
    }
}
