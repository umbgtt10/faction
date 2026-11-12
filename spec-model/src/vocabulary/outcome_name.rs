// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::ToString;
use serde_json::json;
use serde_json::Value;

use crate::command::Command;
use crate::node_state::NodeState;
use crate::vocabulary::variant::Variant;

const CONCLUSION_MODE: &str = "Bootstrapped";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeName {
    ParticipationAccepted,
    ReadyAccepted,
    DuplicateParticipationIgnored,
    DuplicateReadyIgnored,
    NonMemberIgnored,
    LocalParticipationCompleted,
    BroadcastLocalReady,
    AcknowledgeRejoin,
    DeadlineMissed,
    EmitJoinRequest,
    MemberAdmitted,
    DuplicateMemberIgnored,
    JoinDenied,
    Concluded,
}

impl OutcomeName {
    pub const ALL: [OutcomeName; 14] = [
        OutcomeName::ParticipationAccepted,
        OutcomeName::ReadyAccepted,
        OutcomeName::DuplicateParticipationIgnored,
        OutcomeName::DuplicateReadyIgnored,
        OutcomeName::NonMemberIgnored,
        OutcomeName::LocalParticipationCompleted,
        OutcomeName::BroadcastLocalReady,
        OutcomeName::AcknowledgeRejoin,
        OutcomeName::DeadlineMissed,
        OutcomeName::EmitJoinRequest,
        OutcomeName::MemberAdmitted,
        OutcomeName::DuplicateMemberIgnored,
        OutcomeName::JoinDenied,
        OutcomeName::Concluded,
    ];

    #[must_use]
    pub fn of(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|outcome| outcome.name() == name)
            .expect("an outcome")
    }

    #[must_use]
    pub fn encoded(self, command: &Command, state: &NodeState) -> Value {
        match self {
            OutcomeName::LocalParticipationCompleted | OutcomeName::BroadcastLocalReady => {
                Variant::unit(self.name())
            }
            OutcomeName::DeadlineMissed => Variant::of(
                self.name(),
                json!({"confirmed_count": state.collecting_peers.len().to_string()}),
            ),
            OutcomeName::Concluded => {
                Variant::of(self.name(), json!({"mode": Variant::unit(CONCLUSION_MODE)}))
            }
            OutcomeName::ParticipationAccepted
            | OutcomeName::ReadyAccepted
            | OutcomeName::DuplicateParticipationIgnored
            | OutcomeName::DuplicateReadyIgnored
            | OutcomeName::NonMemberIgnored
            | OutcomeName::AcknowledgeRejoin
            | OutcomeName::EmitJoinRequest
            | OutcomeName::MemberAdmitted
            | OutcomeName::DuplicateMemberIgnored
            | OutcomeName::JoinDenied => {
                Variant::of(self.name(), json!({"peer_id": command.peer_id}))
            }
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            OutcomeName::ParticipationAccepted => "ParticipationAccepted",
            OutcomeName::ReadyAccepted => "ReadyAccepted",
            OutcomeName::DuplicateParticipationIgnored => "DuplicateParticipationIgnored",
            OutcomeName::DuplicateReadyIgnored => "DuplicateReadyIgnored",
            OutcomeName::NonMemberIgnored => "NonMemberIgnored",
            OutcomeName::LocalParticipationCompleted => "LocalParticipationCompleted",
            OutcomeName::BroadcastLocalReady => "BroadcastLocalReady",
            OutcomeName::AcknowledgeRejoin => "AcknowledgeRejoin",
            OutcomeName::DeadlineMissed => "DeadlineMissed",
            OutcomeName::EmitJoinRequest => "EmitJoinRequest",
            OutcomeName::MemberAdmitted => "MemberAdmitted",
            OutcomeName::DuplicateMemberIgnored => "DuplicateMemberIgnored",
            OutcomeName::JoinDenied => "JoinDenied",
            OutcomeName::Concluded => "Concluded",
        }
    }
}
