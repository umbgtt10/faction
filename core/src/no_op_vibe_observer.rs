// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::vibe_input::VibeInput;
use crate::vibe_observer::VibeObserver;
use crate::vibe_transition::VibeTransition;

pub struct NoOpVibeObserver;

impl VibeObserver for NoOpVibeObserver {
    fn observe(&mut self, _input: VibeInput, _transition: VibeTransition) {}
}
