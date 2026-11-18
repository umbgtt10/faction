// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::stage::Stage;
use crate::state::State;
use crate::types::Epoch;

use super::reconfiguring::Reconfiguring;

// A concluded node with changes in flight, fewer than its bound (§2): it
// answers as a node at its bound does, and takes another request too.
#[derive(Debug, Clone)]
pub struct Coordinating {
    reconfiguring: Reconfiguring,
}

impl Coordinating {
    #[must_use]
    pub fn new(reconfiguring: Reconfiguring) -> Self {
        Self { reconfiguring }
    }
}

impl State for Coordinating {
    fn step(
        &self,
        command: Command,
        config: &Config,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome>, Stage)> {
        self.reconfiguring
            .requested(&command, config)
            .or_else(|| self.reconfiguring.step(command, config, epoch))
    }

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView {
        self.reconfiguring.cluster_view(previous)
    }
}
