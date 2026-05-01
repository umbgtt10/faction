// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;

pub trait NodeObserver: Send {
    fn on_start(&mut self);

    fn on_step(&mut self, input: &InputMessage, decisions: &[OutputMessage]);

    fn on_idle(&mut self);
}
