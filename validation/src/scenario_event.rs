// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::Freshness;
use faction::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineScenarioEvent {
    ParticipationObserved {
        target_peer_id: PeerId,
        source_peer_id: PeerId,
        freshness: Freshness,
        current_marker: Freshness,
    },
    ReadyObserved {
        target_peer_id: PeerId,
        source_peer_id: PeerId,
        freshness: Freshness,
        current_marker: Freshness,
    },
    LocalParticipationCompleted {
        target_peer_id: PeerId,
    },
    DeadlineExpired {
        target_peer_id: PeerId,
    },
}
