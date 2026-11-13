// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::addition::Addition;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::config::Config;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::types::PeerId;
use serde_json::json;
use serde_json::Value;

// The crate's types written as the specification's JSON: a command, a result
// and a step -- what a vector expects and a log keeps. The admissible commands
// are written by name, so the placeholder peer id the crate gives them is not.
pub struct SpecWriter;

impl SpecWriter {
    pub fn command(command: &Command) -> Value {
        match command {
            Command::ParticipationObserved { peer_id }
            | Command::ReadyObserved { peer_id }
            | Command::JoinRequested { peer_id }
            | Command::JoinApproved { peer_id }
            | Command::JoinRejected { peer_id }
            | Command::AdditionRequested { peer_id }
            | Command::MembershipAcknowledged { peer_id }
            | Command::AdditionCommitted { peer_id }
            | Command::AdditionAborted { peer_id } => {
                json!({ Self::name(command): { "peer_id": Self::number(*peer_id) } })
            }
            Command::RequiredCountChanged { required_count } => {
                json!({ Self::name(command): { "required_count": Self::number(*required_count) } })
            }
            Command::LocalParticipationCompleted | Command::DeadlineExpired | Command::Probe => {
                json!({ Self::name(command): {} })
            }
        }
    }

    pub fn configuration(config: &Config) -> Value {
        json!({
            "peer_id": Self::number(config.peer_id()),
            "peers": Self::peers(config.peers()),
            "required_count": Self::number(config.required_count()),
        })
    }

    pub fn result(result: &ProcessResult) -> Value {
        match result {
            ProcessResult::Accepted {
                cluster_view,
                admissible,
                outcomes,
            } => json!({ "Accepted": {
                "outcomes": outcomes.iter().map(Self::outcome).collect::<Vec<Value>>(),
                "cluster_view": Self::view(cluster_view),
                "admissible": Self::admissible(admissible),
            } }),
            ProcessResult::Rejected {
                cluster_view,
                admissible,
            } => json!({ "Rejected": {
                "cluster_view": Self::view(cluster_view),
                "admissible": Self::admissible(admissible),
            } }),
            ProcessResult::Probed {
                cluster_view,
                admissible,
            } => json!({ "Probed": {
                "cluster_view": Self::view(cluster_view),
                "admissible": Self::admissible(admissible),
            } }),
        }
    }

    pub fn step(command: &Command, result: &ProcessResult) -> Value {
        json!({ "command": Self::command(command), "result": Self::result(result) })
    }

    fn admissible(commands: &[Command]) -> Value {
        Value::Array(
            commands
                .iter()
                .map(|command| Value::from(Self::name(command)))
                .collect(),
        )
    }

    fn name(command: &Command) -> &'static str {
        match command {
            Command::ParticipationObserved { .. } => "ParticipationObserved",
            Command::ReadyObserved { .. } => "ReadyObserved",
            Command::LocalParticipationCompleted => "LocalParticipationCompleted",
            Command::DeadlineExpired => "DeadlineExpired",
            Command::JoinRequested { .. } => "JoinRequested",
            Command::JoinApproved { .. } => "JoinApproved",
            Command::JoinRejected { .. } => "JoinRejected",
            Command::AdditionRequested { .. } => "AdditionRequested",
            Command::MembershipAcknowledged { .. } => "MembershipAcknowledged",
            Command::AdditionCommitted { .. } => "AdditionCommitted",
            Command::AdditionAborted { .. } => "AdditionAborted",
            Command::RequiredCountChanged { .. } => "RequiredCountChanged",
            Command::Probe => "Probe",
        }
    }

    fn number(number: impl ToString) -> Value {
        Value::from(number.to_string())
    }

    fn outcome(outcome: &Outcome) -> Value {
        match outcome {
            Outcome::ParticipationAccepted { peer_id } => {
                Self::peer("ParticipationAccepted", *peer_id)
            }
            Outcome::ReadyAccepted { peer_id } => Self::peer("ReadyAccepted", *peer_id),
            Outcome::DuplicateParticipationIgnored { peer_id } => {
                Self::peer("DuplicateParticipationIgnored", *peer_id)
            }
            Outcome::DuplicateReadyIgnored { peer_id } => {
                Self::peer("DuplicateReadyIgnored", *peer_id)
            }
            Outcome::NonMemberIgnored { peer_id } => Self::peer("NonMemberIgnored", *peer_id),
            Outcome::LocalParticipationCompleted => json!({ "LocalParticipationCompleted": {} }),
            Outcome::BroadcastLocalReady => json!({ "BroadcastLocalReady": {} }),
            Outcome::AcknowledgeRejoin { peer_id } => Self::peer("AcknowledgeRejoin", *peer_id),
            Outcome::DeadlineMissed { confirmed_count } => {
                json!({ "DeadlineMissed": { "confirmed_count": Self::number(confirmed_count) } })
            }
            Outcome::EmitJoinRequest { peer_id } => Self::peer("EmitJoinRequest", *peer_id),
            Outcome::MemberAdmitted { peer_id } => Self::peer("MemberAdmitted", *peer_id),
            Outcome::DuplicateMemberIgnored { peer_id } => {
                Self::peer("DuplicateMemberIgnored", *peer_id)
            }
            Outcome::JoinDenied { peer_id } => Self::peer("JoinDenied", *peer_id),
            Outcome::Concluded { mode } => match mode {
                Conclusion::Bootstrapped => {
                    json!({ "Concluded": { "mode": { "Bootstrapped": {} } } })
                }
            },
            Outcome::SendMembership { peer_id } => Self::peer("SendMembership", *peer_id),
            Outcome::ProposeAddition { peer_id } => Self::peer("ProposeAddition", *peer_id),
            Outcome::AdditionWithdrawn { peer_id } => Self::peer("AdditionWithdrawn", *peer_id),
            Outcome::AcknowledgementIgnored { peer_id } => {
                Self::peer("AcknowledgementIgnored", *peer_id)
            }
            Outcome::AbortIgnored { peer_id } => Self::peer("AbortIgnored", *peer_id),
            Outcome::RequiredCountSet { required_count } => {
                json!({ "RequiredCountSet": { "required_count": Self::number(required_count) } })
            }
        }
    }

    fn addition(addition: Addition) -> Value {
        match addition {
            Addition::None => json!({ "None": {} }),
            Addition::Requested { peer_id } => Self::peer("Requested", peer_id),
            Addition::Acknowledged { peer_id } => Self::peer("Acknowledged", peer_id),
        }
    }

    fn peer(name: &str, peer_id: PeerId) -> Value {
        json!({ name: { "peer_id": Self::number(peer_id) } })
    }

    fn peer_state(state: PeerState) -> Value {
        let name = match state {
            PeerState::Fresh => "Fresh",
            PeerState::Pinging => "Pinging",
            PeerState::Collecting => "Collecting",
            PeerState::Bootstrapped => "Bootstrapped",
            PeerState::TimedOut => "TimedOut",
        };
        json!({ name: {} })
    }

    fn peers(peers: &[PeerId]) -> Value {
        Value::Array(peers.iter().map(|peer| Self::number(*peer)).collect())
    }

    fn view(view: &ClusterView) -> Value {
        json!({
            "peer_state": Self::peer_state(view.peer_state()),
            "is_pinging_completed": view.is_pinging_completed(),
            "pinging_peers": Self::peers(view.pinging_peers()),
            "collecting_peers": Self::peers(view.collecting_peers()),
            "required_count": Self::number(view.required_count()),
            "deadline_missed": view.deadline_missed(),
            "members": Self::peers(view.members().as_slice()),
            "addition": Self::addition(view.addition()),
        })
    }
}
