// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::instance::peer_id::PeerId;
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
pub struct Coordinating<P: PeerId = u64> {
    reconfiguring: Reconfiguring<P>,
}

impl<P: PeerId> Coordinating<P> {
    #[must_use]
    pub fn new(reconfiguring: Reconfiguring<P>) -> Self {
        Self { reconfiguring }
    }
}

impl<P: PeerId> State<P> for Coordinating<P> {
    fn step(
        &self,
        command: Command<P>,
        config: &Config<P>,
        epoch: Epoch,
    ) -> Option<(Vec<Outcome<P>>, Stage<P>)> {
        self.reconfiguring
            .requested(&command, config)
            .or_else(|| self.reconfiguring.step(command, config, epoch))
    }

    fn cluster_view(&self, previous: &ClusterView<P>) -> ClusterView<P> {
        self.reconfiguring.cluster_view(previous)
    }
}
