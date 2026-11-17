// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The property tier, changes in flight: a node of five requiring three, bound
// to one, two or three changes, concluded, then given commands at random.
// It never holds more changes than its bound nor two naming one peer; a
// removal it proposes of a member not suspected leaves the members not
// suspected, less every member whose removal is in flight, at the count or
// above; and it takes a request exactly while concluded below its bound.

use crate::support::random_commands::RandomCommands;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::instance::instance_id::InstanceId;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction::types::PeerId;
use proptest::prelude::*;

// What a call answered: its outcomes, its view and what it admits after.
fn answered(faction: &mut Faction, command: Command) -> (Vec<Outcome>, ClusterView, Vec<Command>) {
    match faction.process(command).output {
        ProcessResult::Accepted {
            outcomes,
            cluster_view,
            admissible,
        } => (outcomes, cluster_view, admissible),
        ProcessResult::Rejected {
            cluster_view,
            admissible,
        }
        | ProcessResult::Probed {
            cluster_view,
            admissible,
        } => (Vec::new(), cluster_view, admissible),
    }
}

// Node 0 of members 0 to 4, requiring three, bound to `bound` changes,
// concluded.
fn concluded(bound: usize) -> Faction {
    let mut faction = Faction::new(
        Config::new(
            InstanceId {
                node: 0,
                incarnation: 1,
            },
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(3),
            1,
            0,
            false,
            bound,
        ),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::LocalParticipationCompleted).output;
    for peer_id in 1..3 {
        let _ = faction
            .process(Command::ReadyObserved { peer_id, epoch: 0 })
            .output;
    }
    faction
}

extern crate alloc;

fn named(change: &Reconfiguration) -> PeerId {
    match change {
        Reconfiguration::AdditionRequested { peer_id }
        | Reconfiguration::AdditionAcknowledged { peer_id }
        | Reconfiguration::RemovalRequested { peer_id } => *peer_id,
    }
}

proptest! {
    #[test]
    fn a_node_never_holds_more_changes_than_its_bound_nor_two_naming_one_peer(
        bound in 1usize..=3,
        commands in RandomCommands::sequence(80),
    ) {
        // Arrange
        let mut faction = concluded(bound);

        // Act
        for command in commands {
            let (_, cluster_view, _) = answered(&mut faction, command);

            // Assert
            let peers: Vec<PeerId> = cluster_view.reconfigurations().iter().map(named).collect();
            prop_assert!(peers.len() <= bound);
            prop_assert!(peers.iter().enumerate().all(|(index, peer)| !peers[index + 1..].contains(peer)));
        }
    }

    #[test]
    fn a_removal_proposed_of_a_member_not_suspected_leaves_the_count_with_the_removals_in_flight(
        bound in 1usize..=3,
        commands in RandomCommands::sequence(80),
    ) {
        // Arrange
        let mut faction = concluded(bound);
        let (_, mut previous, _) = answered(&mut faction, Command::Probe);

        // Act
        for command in commands {
            let (outcomes, cluster_view, _) = answered(&mut faction, command.clone());

            // Assert
            if let Command::RemovalRequested { peer_id } = command
                && outcomes.contains(&Outcome::ProposeRemoval { peer_id })
                && !previous.suspected().contains(&peer_id)
            {
                prop_assert!(spared(&cluster_view));
            }
            previous = cluster_view;
        }
    }

    #[test]
    fn a_node_takes_a_request_exactly_while_concluded_below_its_bound(
        bound in 1usize..=3,
        commands in RandomCommands::sequence(80),
    ) {
        // Arrange
        let mut faction = concluded(bound);

        // Act
        for command in commands {
            let (_, cluster_view, admissible) = answered(&mut faction, command);

            // Assert
            prop_assert_eq!(
                admissible.contains(&Command::AdditionRequested { peer_id: 0 }),
                cluster_view.peer_state() == PeerState::Bootstrapped
                    && cluster_view.reconfigurations().len() < bound
            );
        }
    }
}

// Whether the members not suspected, less every member whose removal is in
// flight, reach the count.
fn spared(cluster_view: &ClusterView) -> bool {
    let removing: Vec<PeerId> = cluster_view
        .reconfigurations()
        .iter()
        .filter_map(|change| match change {
            Reconfiguration::RemovalRequested { peer_id } => Some(*peer_id),
            _ => None,
        })
        .collect();
    cluster_view
        .members()
        .as_slice()
        .iter()
        .filter(|member| !cluster_view.suspected().contains(member) && !removing.contains(member))
        .count()
        >= cluster_view.required_count()
}
