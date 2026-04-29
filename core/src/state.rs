// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::command::Command;
use crate::config::Config;
use crate::outcome::Outcome;
use crate::state_snapshot::StateSnapshot;

pub trait State: StateSnapshot {
    fn step(self: Box<Self>, input: Command, config: &Config) -> (Vec<Outcome>, Box<dyn State>);

    fn accept(&self, _input: &Command) -> bool {
        true
    }
}
