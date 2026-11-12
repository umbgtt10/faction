// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::stage::Stage;

pub trait State: Send {
    fn step(&self, command: Command, config: &Config) -> (Vec<Outcome>, Stage);

    fn cluster_view(&self, previous: &ClusterView) -> ClusterView;

    fn accept(&self, command: &Command) -> bool;
}
