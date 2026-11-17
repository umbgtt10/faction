// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The property tier, epochs (§16): random command sequences, memberships and
// epochs among them, on a node of a five-member cluster created at a random
// epoch. Its epoch never decreases; and a claim from behind -- participation,
// readiness or an addition's acknowledgement at an epoch below the node's --
// never counts: it is never accepted, acknowledged, concluded on or proposed,
// and it leaves the peers the node gathered and its change in flight as they
// were.

use crate::support::random_commands::RandomCommands;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::instance::instance_id::InstanceId;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::types::Epoch;
use proptest::prelude::*;

// The outcomes and the view a command is answered with.
fn answered(faction: &mut Faction, command: Command) -> (Vec<Outcome>, ClusterView) {
    match faction.process(command).output {
        ProcessResult::Accepted {
            outcomes,
            cluster_view,
            ..
        } => (outcomes, cluster_view),
        ProcessResult::Rejected { cluster_view, .. }
        | ProcessResult::Probed { cluster_view, .. } => (Vec::new(), cluster_view),
    }
}

// The epoch a claim carries: participation, readiness or an acknowledgement.
fn claimed(command: &Command) -> Option<Epoch> {
    match *command {
        Command::ParticipationObserved { epoch, .. }
        | Command::ReadyObserved { epoch, .. }
        | Command::MembershipAcknowledged { epoch, .. } => Some(epoch),
        _ => None,
    }
}

// Whether an outcome counts a claim: accepts it, acknowledges it, concludes
// on it or proposes the addition it acknowledges.
fn counts(outcome: &Outcome) -> bool {
    matches!(
        outcome,
        Outcome::ParticipationAccepted { .. }
            | Outcome::ReadyAccepted { .. }
            | Outcome::AcknowledgeRejoin { .. }
            | Outcome::Concluded { .. }
            | Outcome::ProposeAddition { .. }
    )
}

fn faction(genesis: Epoch, required_count: usize) -> Faction {
    Faction::new(
        Config::new(
            InstanceId {
                node: 0,
                incarnation: 1,
            },
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(required_count),
            1,
            genesis,
            false,
        ),
        Box::new(NoOpObserver),
    )
}

proptest! {
    #[test]
    fn process_of_random_commands_never_lowers_the_epoch(
        genesis in 0u64..=2,
        required_count in 0usize..=6,
        commands in RandomCommands::sequence(60),
    ) {
        // Arrange
        let mut faction = faction(genesis, required_count);
        let (_, mut previous) = answered(&mut faction, Command::Probe);

        // Act
        for command in commands {
            let (_, cluster_view) = answered(&mut faction, command);

            // Assert
            prop_assert!(cluster_view.epoch() >= previous.epoch());
            previous = cluster_view;
        }
    }

    #[test]
    fn process_of_a_claim_from_behind_never_counts_it(
        genesis in 0u64..=2,
        required_count in 0usize..=6,
        commands in RandomCommands::sequence(60),
    ) {
        // Arrange
        let mut faction = faction(genesis, required_count);
        let (_, mut previous) = answered(&mut faction, Command::Probe);

        // Act
        for command in commands {
            let behind = claimed(&command).is_some_and(|epoch| epoch < previous.epoch());
            let (outcomes, cluster_view) = answered(&mut faction, command);

            // Assert
            if behind {
                prop_assert!(!outcomes.iter().any(counts), "{outcomes:?}");
                prop_assert_eq!(cluster_view.pinging_peers(), previous.pinging_peers());
                prop_assert_eq!(cluster_view.collecting_peers(), previous.collecting_peers());
                prop_assert_eq!(cluster_view.reconfiguration(), previous.reconfiguration());
            }
            previous = cluster_view;
        }
    }
}
