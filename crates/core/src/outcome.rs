// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::conclusion::Conclusion;
use crate::reconfiguration::Reconfiguration;
use alloc::vec::Vec;

use crate::instance::peer_id::PeerId;
use crate::types::Epoch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome<P: PeerId = u64> {
    ParticipationAccepted { peer_id: P },
    ReadyAccepted { peer_id: P },
    DuplicateParticipationIgnored { peer_id: P },
    DuplicateReadyIgnored { peer_id: P },
    NonMemberIgnored { peer_id: P },
    LocalParticipationCompleted,
    BroadcastLocalReady,
    AcknowledgeRejoin { peer_id: P },
    DeadlineMissed { confirmed_count: usize },
    EmitJoinRequest { peer_id: P },
    MemberAdmitted { peer_id: P },
    DuplicateMemberIgnored { peer_id: P },
    JoinDenied { peer_id: P },
    Concluded { mode: Conclusion },
    SendMembership { peer_id: P },
    ProposeAddition { peer_id: P },
    AdditionWithdrawn { peer_id: P },
    AcknowledgementIgnored { peer_id: P },
    AbortIgnored { peer_id: P },
    DeclineIgnored { peer_id: P },
    ProposeRemoval { peer_id: P },
    RemovalRefused { peer_id: P },
    CoordinationRefused { peer_id: P },
    DuplicateChangeIgnored { peer_id: P },
    RemovalWithdrawn { peer_id: P },
    MemberRemoved { peer_id: P },
    RequiredCountSet { required_count: usize },
    SendProbe { peer_id: P },
    NothingToProbe,
    SendIndirectProbes { peer_id: P, via: Vec<P> },
    MissCounted { peer_id: P, misses: usize },
    PeerSuspected { peer_id: P },
    PeerRevived { peer_id: P },
    AliveNoted { peer_id: P },
    AlreadySuspected { peer_id: P },
    SelfIgnored { peer_id: P },
    QuorumLost,
    QuorumRestored,
    StaleEpochIgnored { peer_id: P, epoch: Epoch },
    CommitIgnored { epoch: Epoch },
    MembershipIgnored { epoch: Epoch },
    MembershipAdopted { epoch: Epoch },
    ChangeAbandoned { reconfiguration: Reconfiguration<P> },
}
