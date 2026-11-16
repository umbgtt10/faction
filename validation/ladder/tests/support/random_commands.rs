// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use proptest::collection::vec;
use proptest::prelude::Just;
use proptest::prop_oneof;
use proptest::sample::subsequence;
use proptest::strategy::Strategy;

// Commands drawn at random for the property and replay tiers: every command,
// its peer among a five-member cluster's ids and two that are not members, a
// new required count from none to more than the cluster holds, an epoch from
// none to a few commits on, and a membership of any of those peers, so every
// row of the decision table can be reached.
pub struct RandomCommands;

impl RandomCommands {
    pub fn command() -> impl Strategy<Value = Command> {
        prop_oneof![
            20 => Self::handed_no_membership(),
            1 => Self::membership(),
        ]
    }

    // Every command but a membership handed to the node: the commands under
    // which what a node gathers only grows but by a removal, and a node that
    // has concluded or been removed stays so.
    pub fn handed_no_membership() -> impl Strategy<Value = Command> {
        let peer = 0u64..=6;
        let epoch = 0u64..=4;
        prop_oneof![
            (peer.clone(), epoch.clone())
                .prop_map(|(peer_id, epoch)| Command::ParticipationObserved { peer_id, epoch }),
            (peer.clone(), epoch.clone())
                .prop_map(|(peer_id, epoch)| Command::ReadyObserved { peer_id, epoch }),
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
            (peer.clone(), epoch.clone())
                .prop_map(|(peer_id, epoch)| Command::MembershipAcknowledged { peer_id, epoch }),
            (peer.clone(), epoch.clone())
                .prop_map(|(peer_id, epoch)| Command::AdditionCommitted { peer_id, epoch }),
            peer.clone()
                .prop_map(|peer_id| Command::AdditionAborted { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::RemovalRequested { peer_id }),
            (peer.clone(), epoch.clone())
                .prop_map(|(peer_id, epoch)| Command::RemovalCommitted { peer_id, epoch }),
            peer.clone()
                .prop_map(|peer_id| Command::RemovalAborted { peer_id }),
            (0usize..=6, epoch).prop_map(|(required_count, epoch)| {
                Command::RequiredCountChanged {
                    required_count,
                    epoch,
                }
            }),
            Just(Command::ProbeDue),
            peer.clone()
                .prop_map(|peer_id| Command::PeerHeard { peer_id }),
            peer.clone()
                .prop_map(|peer_id| Command::ProbeMissed { peer_id }),
            peer.prop_map(|peer_id| Command::IndirectProbesMissed { peer_id }),
            Just(Command::Probe),
        ]
    }

    // A membership of any of the peers, none twice, in any order.
    pub fn membership() -> impl Strategy<Value = Command> {
        (
            subsequence((0u64..=6).collect::<Vec<u64>>(), 0..=7).prop_shuffle(),
            0usize..=6,
            0u64..=4,
        )
            .prop_map(
                |(members, required_count, epoch)| Command::MembershipReceived {
                    members,
                    required_count,
                    epoch,
                },
            )
    }

    pub fn sequence(longest: usize) -> impl Strategy<Value = Vec<Command>> {
        vec(Self::command(), 0..=longest)
    }
}
