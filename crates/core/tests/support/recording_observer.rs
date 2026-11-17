// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::instance::receipt::Receipt;
use faction::observer::Observer;
use std::sync::Arc;
use std::sync::Mutex;

pub type Observations = Arc<Mutex<Vec<(Command, Receipt)>>>;

pub struct RecordingObserver {
    observations: Observations,
}

impl RecordingObserver {
    pub fn new(observations: Observations) -> Self {
        Self { observations }
    }
}

impl Observer for RecordingObserver {
    fn observe(&mut self, command: &Command, receipt: &Receipt) {
        self.observations
            .lock()
            .unwrap()
            .push((command.clone(), receipt.clone()));
    }
}
