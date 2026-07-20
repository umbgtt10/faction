// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::observer::Observer;
use crate::transition::Transition;

pub struct NoOpObserver;

impl Observer for NoOpObserver {
    fn observe(&mut self, _command: Command, _transition: Transition) {}

    fn observe_query(&mut self, _command: Command, _cluster_view: ClusterView) {}

    fn observe_rejection(
        &mut self,
        _command: Command,
        _cluster_view: ClusterView,
        _admissible: Vec<Command>,
    ) {
    }
}
