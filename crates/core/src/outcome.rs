// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::conclusion::Conclusion;
use alloc::vec::Vec;

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
    EmitJoinRequest { peer_id: PeerId },
    MemberAdmitted { peer_id: PeerId },
    DuplicateMemberIgnored { peer_id: PeerId },
    JoinDenied { peer_id: PeerId },
    Concluded { mode: Conclusion },
    SendMembership { peer_id: PeerId },
    ProposeAddition { peer_id: PeerId },
    AdditionWithdrawn { peer_id: PeerId },
    AcknowledgementIgnored { peer_id: PeerId },
    AbortIgnored { peer_id: PeerId },
    ProposeRemoval { peer_id: PeerId },
    RemovalRefused { peer_id: PeerId },
    RemovalWithdrawn { peer_id: PeerId },
    MemberRemoved { peer_id: PeerId },
    RequiredCountSet { required_count: usize },
    SendProbe { peer_id: PeerId },
    NothingToProbe,
    SendIndirectProbes { peer_id: PeerId, via: Vec<PeerId> },
    MissCounted { peer_id: PeerId, misses: usize },
    PeerSuspected { peer_id: PeerId },
    PeerRevived { peer_id: PeerId },
    AliveNoted { peer_id: PeerId },
    AlreadySuspected { peer_id: PeerId },
    SelfIgnored { peer_id: PeerId },
    QuorumLost,
    QuorumRestored,
}
