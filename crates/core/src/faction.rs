// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance::peer_id::PeerId;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::cluster_view_builder::ClusterViewBuilder;
use crate::command::Command;
use crate::command_kind::CommandKind;
use crate::config::Config;
use crate::instance::receipt::Receipt;
use crate::members::Members;
use crate::observer::Observer;
use crate::process_result::ProcessResult;
use crate::stage::Stage;
use crate::state::State;
use crate::states::epoch_rule::EpochRule;
use crate::states::initial::Initial;
use crate::states::provisional_rule::ProvisionalRule;

pub struct Faction<P: PeerId = u64> {
    config: Config<P>,
    observer: Box<dyn Observer<P>>,
    stage: Stage<P>,
    cluster_view: ClusterView<P>,
    next_number: u64,
}

impl<P: PeerId> Faction<P> {
    #[must_use]
    pub fn new(config: Config<P>, observer: Box<dyn Observer<P>>) -> Self {
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
            next_number: 1,
        }
    }

    // The command answered, on a receipt numbered one more than the last,
    // and reported to the observer.
    #[must_use]
    pub fn process(&mut self, command: Command<P>) -> Receipt<P> {
        let receipt = Receipt {
            number: self.next_number,
            output: self.answer(&command),
        };
        self.next_number += 1;
        self.observer.observe(&command, &receipt);
        receipt
    }

    fn admissible(&self) -> Vec<CommandKind> {
        self.stage
            .admissible_commands(&self.config, self.cluster_view.epoch())
    }

    // The node's epoch is its view's, and so is whether its membership is
    // provisional: an accepted command moves each as its rule says, before
    // the stage shows itself; a provisional node sends no membership, and a
    // change it is asked to coordinate it refuses, its stage kept.
    fn answer(&mut self, command: &Command<P>) -> ProcessResult<P> {
        let epoch = self.cluster_view.epoch();
        let provisional =
            ProvisionalRule::new(self.config.peer_id(), self.cluster_view.is_provisional());
        match self.stage.step(command.clone(), &self.config, epoch) {
            Some((outcomes, stage)) => {
                let (outcomes, stage) = provisional.refusal(command).map_or_else(
                    || (provisional.answered(outcomes), stage),
                    |refused| (vec![refused], self.stage.clone()),
                );
                self.stage = stage;
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
            None if command.kind() == CommandKind::Probe => ProcessResult::Probed {
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
