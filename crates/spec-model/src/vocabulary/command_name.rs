// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
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
    RequiredCountChanged,
    MembershipReceived,
    ProbeDue,
    PeerHeard,
    ProbeMissed,
    IndirectProbesMissed,
    Probe,
}

impl CommandName {
    pub const ALL: [CommandName; 21] = [
        CommandName::ParticipationObserved,
        CommandName::ReadyObserved,
        CommandName::LocalParticipationCompleted,
        CommandName::DeadlineExpired,
        CommandName::JoinRequested,
        CommandName::JoinApproved,
        CommandName::JoinRejected,
        CommandName::AdditionRequested,
        CommandName::MembershipAcknowledged,
        CommandName::AdditionCommitted,
        CommandName::AdditionAborted,
        CommandName::RemovalRequested,
        CommandName::RemovalCommitted,
        CommandName::RemovalAborted,
        CommandName::RequiredCountChanged,
        CommandName::MembershipReceived,
        CommandName::ProbeDue,
        CommandName::PeerHeard,
        CommandName::ProbeMissed,
        CommandName::IndirectProbesMissed,
        CommandName::Probe,
    ];

    #[must_use]
    pub fn of(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|command| command.name() == name)
            .expect("a command")
    }

    // Whether the command is a commit of the layer above (§16): an addition's,
    // a removal's or a new count's.
    #[must_use]
    pub fn commits(self) -> bool {
        matches!(
            self,
            CommandName::AdditionCommitted
                | CommandName::RemovalCommitted
                | CommandName::RequiredCountChanged
        )
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            CommandName::ParticipationObserved => "ParticipationObserved",
            CommandName::ReadyObserved => "ReadyObserved",
            CommandName::LocalParticipationCompleted => "LocalParticipationCompleted",
            CommandName::DeadlineExpired => "DeadlineExpired",
            CommandName::JoinRequested => "JoinRequested",
            CommandName::JoinApproved => "JoinApproved",
            CommandName::JoinRejected => "JoinRejected",
            CommandName::AdditionRequested => "AdditionRequested",
            CommandName::MembershipAcknowledged => "MembershipAcknowledged",
            CommandName::AdditionCommitted => "AdditionCommitted",
            CommandName::AdditionAborted => "AdditionAborted",
            CommandName::RemovalRequested => "RemovalRequested",
            CommandName::RemovalCommitted => "RemovalCommitted",
            CommandName::RemovalAborted => "RemovalAborted",
            CommandName::RequiredCountChanged => "RequiredCountChanged",
            CommandName::MembershipReceived => "MembershipReceived",
            CommandName::ProbeDue => "ProbeDue",
            CommandName::PeerHeard => "PeerHeard",
            CommandName::ProbeMissed => "ProbeMissed",
            CommandName::IndirectProbesMissed => "IndirectProbesMissed",
            CommandName::Probe => "Probe",
        }
    }
}
