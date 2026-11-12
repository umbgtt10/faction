// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::observer::Observer;
use faction::process_result::ProcessResult;
use std::sync::Arc;
use std::sync::Mutex;

pub type Observations = Arc<Mutex<Vec<(Command, ProcessResult)>>>;

pub struct RecordingObserver {
    observations: Observations,
}

impl RecordingObserver {
    pub fn new(observations: Observations) -> Self {
        Self { observations }
    }
}

impl Observer for RecordingObserver {
    fn observe(&mut self, command: &Command, result: &ProcessResult) {
        self.observations
            .lock()
            .unwrap()
            .push((*command, result.clone()));
    }
}
