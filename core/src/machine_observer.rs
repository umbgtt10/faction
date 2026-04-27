// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::machine_input::MachineInput;
use crate::machine_transition::MachineTransition;

pub trait MachineObserver {
    fn observe(&mut self, input: MachineInput, transition: MachineTransition);
}
