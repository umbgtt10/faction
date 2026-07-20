// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioEvent {
    ParticipationObserved {
        target_peer_id: PeerId,
        source_peer_id: PeerId,
    },
    ReadyObserved {
        target_peer_id: PeerId,
        source_peer_id: PeerId,
    },
    LocalParticipationCompleted {
        target_peer_id: PeerId,
    },
    DeadlineExpired {
        target_peer_id: PeerId,
    },
}
