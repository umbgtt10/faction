// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::observer::Observer;
use crate::process_result::ProcessResult;
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
        let state: Box<dyn State> = Box::new(Initial::new(Members::new(config.peers().to_vec())));
        let base = ClusterViewBuilder::new()
            .with_required_count(config.required_count())
            .with_members(Members::new(config.peers().to_vec()))
            .build();
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
            let cluster_view = self.cluster_view.clone();
            let admissible = self.state.admissible_commands();
            self.observer.observe_query(command, cluster_view.clone());
            return ProcessResult::Probed {
                cluster_view,
                admissible,
            };
        }

        if !self.state.accept(&command) {
            let cluster_view = self.cluster_view.clone();
            let admissible = self.state.admissible_commands();
            self.observer
                .observe_rejection(command, cluster_view.clone(), admissible.clone());
            return ProcessResult::Rejected {
                cluster_view,
                admissible,
            };
        }

        let previous_cluster_view = self.cluster_view.clone();

        let (outputs, new_state) = self.state.step(command, &self.config);
        self.state = new_state;

        let admissible = self.state.admissible_commands();
        let new_cluster_view = self.state.cluster_view(&previous_cluster_view);
        let transition = Transition::new(
            previous_cluster_view,
            outputs.clone(),
            new_cluster_view.clone(),
        );
        self.observer.observe(command, transition);
        self.cluster_view = new_cluster_view.clone();
        ProcessResult::Accepted {
            cluster_view: new_cluster_view,
            admissible,
            outcomes: outputs,
        }
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}
