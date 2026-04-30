// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;

use crate::command::Command;
use crate::config::Config;
use crate::observer::Observer;
use crate::process_result::ProcessResult;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;
use crate::cluster_view::ClusterView;
use crate::state::State;
use crate::states::initial::Initial;
use crate::transition::Transition;

pub struct Faction {
    config: Config,
    observer: Box<dyn Observer>,
    state: Box<dyn State>,
    cluster_view: ClusterView,
}

impl Faction {
    #[must_use]
    pub fn new(config: Config, observer: Box<dyn Observer>) -> Self {
        let state: Box<dyn State> = Box::new(Initial);
        let base = ClusterView::new(
            ReadinessLifecycleState::Phase1Active,
            false, 0,
            0,
            config.quorum_threshold(),
        );
        let cluster_view = state.cluster_view(&base);
        Self {
            config,
            observer,
            state,
            cluster_view,
        }
    }

    #[must_use]
    pub fn process(&mut self, command: Command) -> ProcessResult {
        if let Command::Probe = command {
            return ProcessResult::Probed {
                cluster_view: self.cluster_view,
                admissible: self.state.admissible_commands(),
            };
        }

        if !self.state.accept(&command) {
            return ProcessResult::Rejected {
                cluster_view: self.cluster_view,
                admissible: self.state.admissible_commands(),
            };
        }

        let previous_snapshot = self.cluster_view;

        let (outputs, new_state) = self.state.step(command, &self.config);
        self.state = new_state;

        let new_snapshot = self.state.cluster_view(&previous_snapshot);
        self.cluster_view = new_snapshot;

        let transition = Transition::new(previous_snapshot, outputs.clone(), new_snapshot);
        self.observer.observe(command, transition);
        ProcessResult::Accepted {
            outcomes: outputs,
            cluster_view: new_snapshot,
        }
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}
