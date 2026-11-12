// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::recording::Recording;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::observer::Observer;
use faction::transition::Transition;
use std::sync::Arc;
use std::sync::Mutex;

pub type Observations = Arc<Mutex<Recording>>;

pub struct RecordingObserver {
    observations: Observations,
}

impl RecordingObserver {
    pub fn new(observations: Observations) -> Self {
        Self { observations }
    }
}

impl Observer for RecordingObserver {
    fn observe(&mut self, command: Command, transition: Transition) {
        self.observations
            .lock()
            .unwrap()
            .transitions
            .push((command, transition));
    }

    fn observe_query(&mut self, command: Command, cluster_view: ClusterView) {
        self.observations
            .lock()
            .unwrap()
            .queries
            .push((command, cluster_view));
    }

    fn observe_rejection(
        &mut self,
        command: Command,
        cluster_view: ClusterView,
        admissible: Vec<Command>,
    ) {
        self.observations
            .lock()
            .unwrap()
            .rejections
            .push((command, cluster_view, admissible));
    }
}
