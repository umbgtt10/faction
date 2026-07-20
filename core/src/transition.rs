// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::outcome::Outcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    previous_view: ClusterView,
    outputs: Vec<Outcome>,
    new_view: ClusterView,
}

impl Transition {
    #[must_use]
    pub fn new(previous_view: ClusterView, outputs: Vec<Outcome>, new_view: ClusterView) -> Self {
        Self {
            previous_view,
            outputs,
            new_view,
        }
    }

    #[must_use]
    pub fn previous_view(&self) -> ClusterView {
        self.previous_view.clone()
    }

    #[must_use]
    pub fn outputs(&self) -> &[Outcome] {
        &self.outputs
    }

    #[must_use]
    pub fn new_view(&self) -> ClusterView {
        self.new_view.clone()
    }
}
