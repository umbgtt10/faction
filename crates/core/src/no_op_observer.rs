// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::observer::Observer;
use crate::process_result::ProcessResult;

pub struct NoOpObserver;

impl Observer for NoOpObserver {
    fn observe(&mut self, _command: &Command, _result: &ProcessResult) {}
}
