// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::conclusion::Conclusion;
use crate::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    ParticipationAccepted { peer_id: PeerId },
    ReadyAccepted { peer_id: PeerId },
    DuplicateParticipationIgnored { peer_id: PeerId },
    DuplicateReadyIgnored { peer_id: PeerId },
    NonMemberIgnored { peer_id: PeerId },
    LocalParticipationCompleted,
    BroadcastLocalReady,
    AcknowledgeRejoin { peer_id: PeerId },
    DeadlineMissed { confirmed_count: usize },
    Concluded { mode: Conclusion },
}
