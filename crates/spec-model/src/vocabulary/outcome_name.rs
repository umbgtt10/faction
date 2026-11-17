// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use serde_json::Value;
use serde_json::json;

use crate::command::Command;
use crate::epoch::Epoch;
use crate::liveness::Liveness;
use crate::node_state::NodeState;
use crate::view::View;
use crate::vocabulary::command_name::CommandName;
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
    DeclineIgnored,
    ProposeRemoval,
    RemovalRefused,
    CoordinationRefused,
    DuplicateChangeIgnored,
    RemovalWithdrawn,
    MemberRemoved,
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
    StaleEpochIgnored,
    CommitIgnored,
    MembershipIgnored,
    MembershipAdopted,
    ChangeAbandoned,
}

impl OutcomeName {
    pub const ALL: [OutcomeName; 43] = [
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
        OutcomeName::DeclineIgnored,
        OutcomeName::ProposeRemoval,
        OutcomeName::RemovalRefused,
        OutcomeName::CoordinationRefused,
        OutcomeName::DuplicateChangeIgnored,
        OutcomeName::RemovalWithdrawn,
        OutcomeName::MemberRemoved,
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
        OutcomeName::StaleEpochIgnored,
        OutcomeName::CommitIgnored,
        OutcomeName::MembershipIgnored,
        OutcomeName::MembershipAdopted,
        OutcomeName::ChangeAbandoned,
    ];

    #[must_use]
    pub fn of(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|outcome| outcome.name() == name)
            .expect("an outcome")
    }

    // The outcome as the receipt writes it: for ChangeAbandoned one for each
    // change in flight, in the order requested, carrying it as the view shows
    // it (§16); for any other, one.
    #[must_use]
    pub fn encoded(self, command: &Command, state: &NodeState) -> Vec<Value> {
        match self {
            OutcomeName::ChangeAbandoned => state
                .reconfigurations
                .iter()
                .map(|change| {
                    Variant::of(
                        self.name(),
                        json!({"reconfiguration": View::reconfiguration(change)}),
                    )
                })
                .collect(),
            _ => vec![self.single(command, state)],
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
            OutcomeName::DeclineIgnored => "DeclineIgnored",
            OutcomeName::ProposeRemoval => "ProposeRemoval",
            OutcomeName::RemovalRefused => "RemovalRefused",
            OutcomeName::CoordinationRefused => "CoordinationRefused",
            OutcomeName::DuplicateChangeIgnored => "DuplicateChangeIgnored",
            OutcomeName::RemovalWithdrawn => "RemovalWithdrawn",
            OutcomeName::MemberRemoved => "MemberRemoved",
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
            OutcomeName::StaleEpochIgnored => "StaleEpochIgnored",
            OutcomeName::CommitIgnored => "CommitIgnored",
            OutcomeName::MembershipIgnored => "MembershipIgnored",
            OutcomeName::MembershipAdopted => "MembershipAdopted",
            OutcomeName::ChangeAbandoned => "ChangeAbandoned",
        }
    }

    fn named(command: &Command) -> &str {
        command.peer_id.as_deref().expect("a peer")
    }

    // The peer a removal names: the command's, or, for a membership that does
    // not name the node, the node itself (§16).
    fn removed<'a>(command: &'a Command, state: &'a NodeState) -> &'a str {
        if command.name == CommandName::MembershipReceived {
            &state.peer_id
        } else {
            Self::named(command)
        }
    }

    // Any other outcome as the receipt writes it: every outcome not named here
    // names the command's peer.
    fn single(self, command: &Command, state: &NodeState) -> Value {
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
            OutcomeName::StaleEpochIgnored => Variant::of(
                self.name(),
                Epoch::stated(json!({"peer_id": command.peer_id}), command.epoch),
            ),
            OutcomeName::CommitIgnored
            | OutcomeName::MembershipIgnored
            | OutcomeName::MembershipAdopted => {
                Variant::of(self.name(), Epoch::stated(json!({}), command.epoch))
            }
            OutcomeName::MemberRemoved => Variant::of(
                self.name(),
                json!({"peer_id": Self::removed(command, state)}),
            ),
            OutcomeName::RequiredCountSet => Variant::of(
                self.name(),
                json!({"required_count": command.required_count.map(|count| count.to_string())}),
            ),
            _ => Variant::of(self.name(), json!({"peer_id": command.peer_id})),
        }
    }
}
