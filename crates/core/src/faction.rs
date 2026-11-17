// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::config::Config;
use crate::members::Members;
use crate::observer::Observer;
use crate::process_result::ProcessResult;
use crate::stage::Stage;
use crate::state::State;
use crate::states::epoch_rule::EpochRule;
use crate::states::initial::Initial;
use crate::states::provisional_rule::ProvisionalRule;

pub struct Faction {
    config: Config,
    observer: Box<dyn Observer>,
    stage: Stage,
    cluster_view: ClusterView,
}

impl Faction {
    #[must_use]
    pub fn new(config: Config, observer: Box<dyn Observer>) -> Self {
        let stage = Stage::Initial(Initial::new(
            Members::new(config.peers().to_vec()),
            config.quorum_policy(),
        ));
        let genesis = ClusterViewBuilder::default()
            .with_required_count(config.required_count())
            .with_members(Members::new(config.peers().to_vec()))
            .build();
        let base =
            ClusterViewBuilder::from_view_at(&genesis, config.epoch(), config.is_provisional())
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
        let result = self.answer(&command);
        self.observer.observe(&command, &result);
        result
    }

    fn admissible(&self) -> Vec<Command> {
        self.stage
            .admissible_commands(&self.config, self.cluster_view.epoch())
    }

    // The node's epoch is its view's, and so is whether its membership is
    // provisional: an accepted command moves each as its rule says, before
    // the stage shows itself, and a provisional node sends no membership.
    fn answer(&mut self, command: &Command) -> ProcessResult {
        let epoch = self.cluster_view.epoch();
        let provisional =
            ProvisionalRule::new(self.config.peer_id(), self.cluster_view.is_provisional());
        match self.stage.step(command.clone(), &self.config, epoch) {
            Some((outcomes, stage)) => {
                self.stage = stage;
                let outcomes = provisional.answered(outcomes);
                let moved = ClusterViewBuilder::from_view_at(
                    &self.cluster_view,
                    EpochRule::new(epoch).after(command),
                    provisional.after(command, &outcomes),
                )
                .build();
                self.cluster_view = self.stage.cluster_view(&moved);
                ProcessResult::Accepted {
                    cluster_view: self.cluster_view.clone(),
                    admissible: self.admissible(),
                    outcomes,
                }
            }
            None if *command == Command::Probe => ProcessResult::Probed {
                cluster_view: self.cluster_view.clone(),
                admissible: self.admissible(),
            },
            None => ProcessResult::Rejected {
                cluster_view: self.cluster_view.clone(),
                admissible: self.admissible(),
            },
        }
    }
}
