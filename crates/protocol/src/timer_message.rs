// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::instance::peer_id::PeerId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimerMessage<P: PeerId = u64> {
    ParticipationObserved { peer_id: P },
    LocalParticipationCompleted,
    RetryPing,
    RetryReady,
    DeadlineExpired,
    RetryMembership { peer_id: P },
    ProbeDue,
    ProbeMissed { peer_id: P },
    IndirectProbesMissed { peer_id: P },
}
