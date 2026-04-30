// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::outcome::Outcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    previous_state: ClusterView,
    outputs: Vec<Outcome>,
    new_state: ClusterView,
}

impl Transition {
    #[must_use]
    pub fn new(previous_state: ClusterView, outputs: Vec<Outcome>, new_state: ClusterView) -> Self {
        Self {
            previous_state,
            outputs,
            new_state,
        }
    }

    #[must_use]
    pub const fn previous_state(&self) -> ClusterView {
        self.previous_state
    }

    #[must_use]
    pub fn outputs(&self) -> &[Outcome] {
        &self.outputs
    }

    #[must_use]
    pub const fn new_state(&self) -> ClusterView {
        self.new_state
    }
}
