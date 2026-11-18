// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::config::Config;
use faction::instance::instance_id::InstanceId;
use faction::quorum_policy::QuorumPolicy;
use serde_json::Value;

// The specification's JSON read into the crate's types: a configuration and a
// command, as a vector or a log gives them.
pub struct SpecReader;

impl SpecReader {
    pub fn command(value: &Value) -> Command {
        let (name, body) = value
            .as_object()
            .and_then(|object| object.iter().next())
            .expect("a command is an object of one member");
        let peer_id = || Self::number(&body["peer_id"]);
        let epoch = || body.get("epoch").map_or(0, Self::number);
        let required_count = || {
            usize::try_from(Self::number(&body["required_count"]))
                .expect("a required count this platform can hold")
        };
        match name.as_str() {
            "ParticipationObserved" => Command::ParticipationObserved {
                peer_id: peer_id(),
                epoch: epoch(),
            },
            "ReadyObserved" => Command::ReadyObserved {
                peer_id: peer_id(),
                epoch: epoch(),
            },
            "LocalParticipationCompleted" => Command::LocalParticipationCompleted,
            "DeadlineExpired" => Command::DeadlineExpired,
            "JoinRequested" => Command::JoinRequested { peer_id: peer_id() },
            "JoinApproved" => Command::JoinApproved { peer_id: peer_id() },
            "JoinRejected" => Command::JoinRejected { peer_id: peer_id() },
            "AdditionRequested" => Command::AdditionRequested { peer_id: peer_id() },
            "MembershipAcknowledged" => Command::MembershipAcknowledged {
                peer_id: peer_id(),
                epoch: epoch(),
            },
            "AdditionCommitted" => Command::AdditionCommitted {
                peer_id: peer_id(),
                epoch: epoch(),
            },
            "AdditionAborted" => Command::AdditionAborted { peer_id: peer_id() },
            "RemovalRequested" => Command::RemovalRequested { peer_id: peer_id() },
            "RemovalCommitted" => Command::RemovalCommitted {
                peer_id: peer_id(),
                epoch: epoch(),
            },
            "RemovalAborted" => Command::RemovalAborted { peer_id: peer_id() },
            "ProposalDeclined" => Command::ProposalDeclined {
                peer_id: peer_id(),
                epoch: epoch(),
            },
            "RequiredCountChanged" => Command::RequiredCountChanged {
                required_count: required_count(),
                epoch: epoch(),
            },
            "MembershipReceived" => Command::MembershipReceived {
                members: Self::peers(&body["members"]),
                required_count: required_count(),
                epoch: epoch(),
            },
            "ProbeDue" => Command::ProbeDue,
            "PeerHeard" => Command::PeerHeard { peer_id: peer_id() },
            "ProbeMissed" => Command::ProbeMissed { peer_id: peer_id() },
            "IndirectProbesMissed" => Command::IndirectProbesMissed { peer_id: peer_id() },
            "Probe" => Command::Probe,
            other => panic!("not a command of the specification: {other}"),
        }
    }

    pub fn configuration(value: &Value) -> Config {
        let peers = Self::peers(&value["peers"]);
        let required = usize::try_from(Self::number(&value["required_count"]))
            .expect("a required count this platform can hold");
        let threshold = usize::try_from(Self::number(&value["suspicion_threshold"]))
            .expect("a suspicion threshold this platform can hold");
        Config::new(
            InstanceId {
                node: Self::number(&value["instance"]["node"]),
                incarnation: Self::number(&value["instance"]["incarnation"]),
            },
            peers,
            QuorumPolicy::new(required),
            threshold,
            value.get("epoch").map_or(0, Self::number),
            value
                .get("provisional")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            value
                .get("changes_in_flight")
                .map_or(Ok(1), |bound| usize::try_from(Self::number(bound)))
                .expect("a bound this platform can hold"),
        )
    }

    fn peers(value: &Value) -> Vec<u64> {
        value
            .as_array()
            .expect("a list of peers")
            .iter()
            .map(Self::number)
            .collect()
    }

    fn number(value: &Value) -> u64 {
        value
            .as_str()
            .and_then(|text| text.parse().ok())
            .expect("a number as a string of decimal digits")
    }
}
