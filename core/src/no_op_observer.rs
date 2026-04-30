// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::command::Command;
use crate::observer::Observer;
use crate::transition::Transition;

pub struct NoOpObserver;

impl Observer for NoOpObserver {
    fn observe(&mut self, _command: Command, _transition: Transition) {}
}
