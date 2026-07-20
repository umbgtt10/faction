// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::outcome::Outcome;
use faction::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioTraceEntry {
    peer_id: PeerId,
    command: Command,
    outputs: Vec<Outcome>,
    cluster_view: ClusterView,
}

impl ScenarioTraceEntry {
    #[must_use]
    pub fn new(
        peer_id: PeerId,
        command: Command,
        outputs: Vec<Outcome>,
        cluster_view: ClusterView,
    ) -> Self {
        Self {
            peer_id,
            command,
            outputs,
            cluster_view,
        }
    }

    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    #[must_use]
    pub const fn command(&self) -> Command {
        self.command
    }

    #[must_use]
    pub fn outputs(&self) -> &[Outcome] {
        &self.outputs
    }

    #[must_use]
    pub fn cluster_view(&self) -> ClusterView {
        self.cluster_view.clone()
    }
}
