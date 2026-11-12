// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::process_result::ProcessResult;

pub trait Observer {
    fn observe(&mut self, command: &Command, result: &ProcessResult);
}
