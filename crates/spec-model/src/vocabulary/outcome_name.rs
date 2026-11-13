// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::ToString;
use serde_json::json;
use serde_json::Value;

use crate::command::Command;
use crate::liveness::Liveness;
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
    SendMembership,
    ProposeAddition,
    AdditionWithdrawn,
    AcknowledgementIgnored,
    AbortIgnored,
    RequiredCountSet,
    SendProbe,
    NothingToProbe,
    SendIndirectProbes,
    MissCounted,
    PeerSuspected,
    PeerRevived,
    AliveNoted,
    AlreadySuspected,
    SelfIgnored,
    QuorumLost,
    QuorumRestored,
}

impl OutcomeName {
    pub const ALL: [OutcomeName; 31] = [
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
        OutcomeName::SendMembership,
        OutcomeName::ProposeAddition,
        OutcomeName::AdditionWithdrawn,
        OutcomeName::AcknowledgementIgnored,
        OutcomeName::AbortIgnored,
        OutcomeName::RequiredCountSet,
        OutcomeName::SendProbe,
        OutcomeName::NothingToProbe,
        OutcomeName::SendIndirectProbes,
        OutcomeName::MissCounted,
        OutcomeName::PeerSuspected,
        OutcomeName::PeerRevived,
        OutcomeName::AliveNoted,
        OutcomeName::AlreadySuspected,
        OutcomeName::SelfIgnored,
        OutcomeName::QuorumLost,
        OutcomeName::QuorumRestored,
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
            OutcomeName::LocalParticipationCompleted
            | OutcomeName::BroadcastLocalReady
            | OutcomeName::NothingToProbe
            | OutcomeName::QuorumLost
            | OutcomeName::QuorumRestored => Variant::unit(self.name()),
            OutcomeName::SendProbe => Variant::of(
                self.name(),
                json!({"peer_id": Liveness::next_to_probe(state)}),
            ),
            OutcomeName::SendIndirectProbes => Variant::of(
                self.name(),
                json!({
                    "peer_id": command.peer_id,
                    "via": Liveness::helpers(state, Self::named(command)),
                }),
            ),
            OutcomeName::MissCounted => Variant::of(
                self.name(),
                json!({
                    "peer_id": command.peer_id,
                    "misses": (Liveness::misses(state, Self::named(command)) + 1).to_string(),
                }),
            ),
            OutcomeName::DeadlineMissed => Variant::of(
                self.name(),
                json!({"confirmed_count": state.collecting_peers.len().to_string()}),
            ),
            OutcomeName::Concluded => {
                Variant::of(self.name(), json!({"mode": Variant::unit(CONCLUSION_MODE)}))
            }
            OutcomeName::RequiredCountSet => Variant::of(
                self.name(),
                json!({"required_count": command.required_count.map(|count| count.to_string())}),
            ),
            OutcomeName::ParticipationAccepted
            | OutcomeName::ReadyAccepted
            | OutcomeName::DuplicateParticipationIgnored
            | OutcomeName::DuplicateReadyIgnored
            | OutcomeName::NonMemberIgnored
            | OutcomeName::AcknowledgeRejoin
            | OutcomeName::EmitJoinRequest
            | OutcomeName::MemberAdmitted
            | OutcomeName::DuplicateMemberIgnored
            | OutcomeName::JoinDenied
            | OutcomeName::SendMembership
            | OutcomeName::ProposeAddition
            | OutcomeName::AdditionWithdrawn
            | OutcomeName::AcknowledgementIgnored
            | OutcomeName::AbortIgnored
            | OutcomeName::PeerSuspected
            | OutcomeName::PeerRevived
            | OutcomeName::AliveNoted
            | OutcomeName::AlreadySuspected
            | OutcomeName::SelfIgnored => {
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
            OutcomeName::SendMembership => "SendMembership",
            OutcomeName::ProposeAddition => "ProposeAddition",
            OutcomeName::AdditionWithdrawn => "AdditionWithdrawn",
            OutcomeName::AcknowledgementIgnored => "AcknowledgementIgnored",
            OutcomeName::AbortIgnored => "AbortIgnored",
            OutcomeName::RequiredCountSet => "RequiredCountSet",
            OutcomeName::SendProbe => "SendProbe",
            OutcomeName::NothingToProbe => "NothingToProbe",
            OutcomeName::SendIndirectProbes => "SendIndirectProbes",
            OutcomeName::MissCounted => "MissCounted",
            OutcomeName::PeerSuspected => "PeerSuspected",
            OutcomeName::PeerRevived => "PeerRevived",
            OutcomeName::AliveNoted => "AliveNoted",
            OutcomeName::AlreadySuspected => "AlreadySuspected",
            OutcomeName::SelfIgnored => "SelfIgnored",
            OutcomeName::QuorumLost => "QuorumLost",
            OutcomeName::QuorumRestored => "QuorumRestored",
        }
    }

    fn named(command: &Command) -> &str {
        command.peer_id.as_deref().expect("a peer")
    }
}
