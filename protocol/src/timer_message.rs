// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimerMessage {
    ParticipationObserved { peer_id: PeerId },
    LocalParticipationCompleted,
    RetryPing,
    RetryReady,
    DeadlineExpired,
}
