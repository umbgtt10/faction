// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;

pub trait NodeObserver: Send {
    fn on_start(&mut self);

    fn on_step(&mut self, input: &InputMessage, decisions: &[OutputMessage]);

    fn on_idle(&mut self);
}
