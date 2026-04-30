// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::apply_status::ApplyStatus;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::outcome::Outcome;
use faction::snapshot::Snapshot;
use faction::PeerId;

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
    pub fn snapshot(&self) -> Snapshot {
        self.readiness.snapshot()
    }

    #[must_use]
    pub fn apply(self, input: faction::command::Command) -> (Self, Vec<Outcome>) {
        let mut readiness = self.readiness;
        let outputs = match readiness.apply(input) {
            ApplyStatus::Accepted { outcomes, .. } => outcomes,
            ApplyStatus::Snapshot { .. } => unreachable!(),
            ApplyStatus::Rejected { .. } => Vec::new(),
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
