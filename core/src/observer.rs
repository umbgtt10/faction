// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::command::Command;
use crate::transition::Transition;

pub trait Observer {
    fn observe(&mut self, input: Command, transition: Transition);
}
