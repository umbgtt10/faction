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
use crate::stage::Stage;
use crate::state::State;
use crate::states::initial::Initial;

pub struct Faction {
    config: Config,
    observer: Box<dyn Observer>,
    stage: Stage,
    cluster_view: ClusterView,
}

impl Faction {
    #[must_use]
    pub fn new(config: Config, observer: Box<dyn Observer>) -> Self {
        let stage = Stage::Initial(Initial::new(Members::new(config.peers().to_vec())));
        let base = ClusterViewBuilder::default()
            .with_required_count(config.required_count())
            .with_members(Members::new(config.peers().to_vec()))
            .build();
        let cluster_view = stage.cluster_view(&base);
        Self {
            config,
            observer,
            stage,
            cluster_view,
        }
    }

    #[must_use]
    pub fn process(&mut self, command: Command) -> ProcessResult {
        let result = self.answer(command);
        self.observer.observe(&command, &result);
        result
    }

    fn answer(&mut self, command: Command) -> ProcessResult {
        match self.stage.step(command, &self.config) {
            Some((outcomes, stage)) => {
                self.stage = stage;
                self.cluster_view = self.stage.cluster_view(&self.cluster_view);
                ProcessResult::Accepted {
                    cluster_view: self.cluster_view.clone(),
                    admissible: self.stage.admissible_commands(&self.config),
                    outcomes,
                }
            }
            None if command == Command::Probe => ProcessResult::Probed {
                cluster_view: self.cluster_view.clone(),
                admissible: self.stage.admissible_commands(&self.config),
            },
            None => ProcessResult::Rejected {
                cluster_view: self.cluster_view.clone(),
                admissible: self.stage.admissible_commands(&self.config),
            },
        }
    }
}
