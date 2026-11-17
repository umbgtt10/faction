// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::instance::receipt::Receipt;

pub trait Observer {
    fn observe(&mut self, command: &Command, receipt: &Receipt);
}
