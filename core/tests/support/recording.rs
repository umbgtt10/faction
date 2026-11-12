// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::transition::Transition;

#[derive(Default)]
pub struct Recording {
    pub transitions: Vec<(Command, Transition)>,
    pub queries: Vec<(Command, ClusterView)>,
    pub rejections: Vec<(Command, ClusterView, Vec<Command>)>,
}
