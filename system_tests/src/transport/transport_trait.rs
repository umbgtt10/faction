// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;
use faction::command::Command;

pub trait Transport: Send {
    fn send(&mut self, to: PeerId, message: Command);

    fn recv(&mut self) -> Option<(PeerId, Command)>;
}
