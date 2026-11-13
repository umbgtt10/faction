// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::types::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    ParticipationObserved { peer_id: PeerId },
    ReadyObserved { peer_id: PeerId },
    LocalParticipationCompleted,
    DeadlineExpired,
    JoinRequested { peer_id: PeerId },
    JoinApproved { peer_id: PeerId },
    JoinRejected { peer_id: PeerId },
    AdditionRequested { peer_id: PeerId },
    MembershipAcknowledged { peer_id: PeerId },
    AdditionCommitted { peer_id: PeerId },
    AdditionAborted { peer_id: PeerId },
    RequiredCountChanged { required_count: usize },
    Probe,
}
