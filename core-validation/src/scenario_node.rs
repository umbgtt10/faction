// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::types::PeerId;

pub struct ScenarioNode {
    peer_id: PeerId,
    readiness: Faction,
}

impl ScenarioNode {
    #[must_use]
    pub fn new(peer_id: PeerId, config: Config) -> Self {
        Self {
            peer_id,
            readiness: Faction::new(config, Box::new(NoOpObserver)),
        }
    }

    #[must_use]
    pub fn new_with_observer(peer_id: PeerId, config: Config, observer: Box<dyn Observer>) -> Self {
        Self {
            peer_id,
            readiness: Faction::new(config, observer),
        }
    }

    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        self.readiness.config()
    }

    #[must_use]
    pub fn cluster_view(&mut self) -> ClusterView {
        match self.readiness.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn process(self, command: Command) -> (Self, Vec<Outcome>) {
        let mut readiness = self.readiness;
        let outputs = match readiness.process(command) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => Vec::new(),
        };

        (
            Self {
                peer_id: self.peer_id,
                readiness,
            },
            outputs,
        )
    }
}
