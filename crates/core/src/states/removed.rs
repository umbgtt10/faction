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
use crate::reconfiguration::Reconfiguration;
use crate::stage::Stage;
use crate::state::State;
use crate::types::PeerId;

// A node removed from its own cluster: it admits a probe alone, and shows what
// it held when it was removed, but itself and its liveness.
#[derive(Debug, Clone, Copy)]
pub struct Removed {
    peer_id: PeerId,
}

impl Removed {
    #[must_use]
    pub fn new(peer_id: PeerId) -> Self {
        Self { peer_id }
    }
}

impl State for Removed {
    fn step(&self, _command: Command, _config: &Config) -> Option<(Vec<Outcome>, Stage)> {
        None
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        let members = Members::new(
            previous
                .members()
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
            .with_members(members)
            .with_reconfiguration(Reconfiguration::None)
            .with_suspected(Vec::new())
            .with_misses(Vec::new())
            .with_last_probed(None)
            .with_quorum(Quorum::NotTracked)
            .build()
    }
}
