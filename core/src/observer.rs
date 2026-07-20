// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::transition::Transition;

pub trait Observer {
    fn observe(&mut self, command: Command, transition: Transition);

    fn observe_query(&mut self, command: Command, cluster_view: ClusterView);

    fn observe_rejection(
        &mut self,
        command: Command,
        cluster_view: ClusterView,
        admissible: Vec<Command>,
    );
}
