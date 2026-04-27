// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::machine_input::MachineInput;
use crate::machine_observer::MachineObserver;
use crate::machine_transition::MachineTransition;

pub struct NoOpMachineObserver;

impl MachineObserver for NoOpMachineObserver {
    fn observe(&mut self, _input: MachineInput, _transition: MachineTransition) {}
}
