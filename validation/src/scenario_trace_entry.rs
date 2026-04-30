// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::outcome::Outcome;
use faction::PeerId;

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
