// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::vibe_config::VibeConfig;
use crate::vibe_input::VibeInput;
use crate::vibe_output::VibeOutput;
use crate::vibe_snapshot::VibeSnapshot;

pub trait VibeState {
    fn punch(
        self: Box<Self>,
        input: VibeInput,
        config: &VibeConfig,
    ) -> (Vec<VibeOutput>, Box<dyn VibeState>);

    fn vibe_check(&self, quorum_threshold: usize) -> VibeSnapshot;

    fn deal(&self, input: &VibeInput) -> bool;
}
