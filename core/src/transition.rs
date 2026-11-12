// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::outcome::Outcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub previous_view: ClusterView,
    pub outputs: Vec<Outcome>,
    pub new_view: ClusterView,
}
