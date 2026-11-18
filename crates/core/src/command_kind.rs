// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// A command named by its kind alone, without the peer or the epoch it
// carries: what a stage admits, as the specification writes an admissible
// command, by its name (§4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    ParticipationObserved,
    ReadyObserved,
    LocalParticipationCompleted,
    DeadlineExpired,
    JoinRequested,
    JoinApproved,
    JoinRejected,
    AdditionRequested,
    MembershipAcknowledged,
    AdditionCommitted,
    AdditionAborted,
    RemovalRequested,
    RemovalCommitted,
    RemovalAborted,
    ProposalDeclined,
    RequiredCountChanged,
    MembershipReceived,
    ProbeDue,
    PeerHeard,
    ProbeMissed,
    IndirectProbesMissed,
    Probe,
}
