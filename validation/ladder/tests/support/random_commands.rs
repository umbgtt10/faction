// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use proptest::collection::vec;
use proptest::prelude::Just;
use proptest::prop_oneof;
use proptest::strategy::Strategy;

// Commands drawn at random for the property and replay tiers: every command,
// its peer among a five-member cluster's ids and two that are not members, and a
// new required count from none to more than the cluster holds, so every row of
// the decision table can be reached.
pub struct RandomCommands;

impl RandomCommands {
    pub fn command() -> impl Strategy<Value = Command> {
        let peer = 0u64..=6;
        prop_oneof![
            peer.clone()
                .prop_map(|peer_id| Command::ParticipationObserved { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::ReadyObserved { peer_id }),
            Just(Command::LocalParticipationCompleted),
            Just(Command::DeadlineExpired),
            peer.clone()
                .prop_map(|peer_id| Command::JoinRequested { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::JoinApproved { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::JoinRejected { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::AdditionRequested { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::MembershipAcknowledged { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::AdditionCommitted { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::AdditionAborted { peer_id }),
            (0usize..=6)
                .prop_map(|required_count| Command::RequiredCountChanged { required_count }),
            Just(Command::ProbeDue),
            peer.clone()
                .prop_map(|peer_id| Command::PeerHeard { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::ProbeMissed { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::IndirectProbesMissed { peer_id }),
            Just(Command::Probe),
        ]
    }

    pub fn sequence(longest: usize) -> impl Strategy<Value = Vec<Command>> {
        vec(Self::command(), 0..=longest)
    }
}
