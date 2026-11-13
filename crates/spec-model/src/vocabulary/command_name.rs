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
    RequiredCountChanged,
    Probe,
}

impl CommandName {
    pub const ALL: [CommandName; 13] = [
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
        CommandName::RequiredCountChanged,
        CommandName::Probe,
    ];

    #[must_use]
    pub fn of(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|command| command.name() == name)
            .expect("a command")
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
            CommandName::RequiredCountChanged => "RequiredCountChanged",
            CommandName::Probe => "Probe",
        }
    }
}
