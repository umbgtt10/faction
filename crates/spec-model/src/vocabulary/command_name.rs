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
    Probe,
}

impl CommandName {
    pub const ALL: [CommandName; 8] = [
        CommandName::ParticipationObserved,
        CommandName::ReadyObserved,
        CommandName::LocalParticipationCompleted,
        CommandName::DeadlineExpired,
        CommandName::JoinRequested,
        CommandName::JoinApproved,
        CommandName::JoinRejected,
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
            CommandName::Probe => "Probe",
        }
    }
}
