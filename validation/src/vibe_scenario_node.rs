// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;

use faction::no_op_vibe_observer::NoOpVibeObserver;
use faction::vibe::Vibe;
use faction::vibe_config::VibeConfig;
use faction::vibe_observer::VibeObserver;
use faction::vibe_output::VibeOutput;
use faction::vibe_snapshot::VibeSnapshot;
use faction::PeerId;

pub struct VibeScenarioNode {
    peer_id: PeerId,
    readiness: Vibe,
}

impl VibeScenarioNode {
    #[must_use]
    pub fn new(peer_id: PeerId, config: VibeConfig) -> Self {
        Self {
            peer_id,
            readiness: Vibe::new(config, Box::new(NoOpVibeObserver)),
        }
    }

    #[must_use]
    pub fn new_with_observer(
        peer_id: PeerId,
        config: VibeConfig,
        observer: Box<dyn VibeObserver>,
    ) -> Self {
        Self {
            peer_id,
            readiness: Vibe::new(config, observer),
        }
    }

    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    #[must_use]
    pub fn config(&self) -> &VibeConfig {
        self.readiness.config()
    }

    #[must_use]
    pub fn snapshot(&self) -> VibeSnapshot {
        self.readiness.snapshot()
    }

    #[must_use]
    pub fn apply(self, input: faction::vibe_input::VibeInput) -> (Self, Vec<VibeOutput>) {
        let mut readiness = self.readiness;
        let outputs = readiness.apply(input);

        (
            Self {
                peer_id: self.peer_id,
                readiness,
            },
            outputs,
        )
    }
}
