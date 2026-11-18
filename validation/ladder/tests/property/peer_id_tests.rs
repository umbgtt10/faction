// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The crate on peer ids of its own type: random command sequences answered by
// a node on u64 ids and by one on the same ids renamed one to one into bytes,
// every answer printed alike -- the machine compares peer ids, and never
// reads one.

use crate::support::random_commands::RandomCommands;
use crate::support::renamed_peer::RenamedPeer;
use faction::config::Config;
use faction::faction::Faction;
use faction::instance::instance_id::InstanceId;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;
use proptest::prelude::*;

const PEERS: [u64; 5] = [0, 1, 2, 3, 4];

proptest! {
    #[test]
    fn process_of_random_commands_on_renamed_peer_ids_answers_alike(
        required_count in 0usize..=6,
        threshold in 1usize..=3,
        bound in 1usize..=3,
        commands in RandomCommands::sequence(40),
    ) {
        // Arrange
        let mut numbered: Faction = Faction::new(
            Config::new(InstanceId { node: 0, incarnation: 1 }, PEERS.to_vec(), QuorumPolicy::new(required_count), threshold, 0, false, bound),
            Box::new(NoOpObserver),
        );
        let mut renamed = Faction::new(
            Config::new(InstanceId { node: RenamedPeer::new(0), incarnation: 1 }, PEERS.map(RenamedPeer::new).to_vec(), QuorumPolicy::new(required_count), threshold, 0, false, bound),
            Box::new(NoOpObserver),
        );

        // Act
        let departure = commands.iter().enumerate().find_map(|(index, command)| {
            let answered = format!("{:?}", numbered.process(command.clone()));
            let renamed_answer = format!("{:?}", renamed.process(RenamedPeer::command(command)));
            (answered != renamed_answer).then(|| {
                format!("call {}, {command:?}: on u64 {answered}, renamed {renamed_answer}", index + 1)
            })
        });

        // Assert
        prop_assert!(departure.is_none(), "{}", departure.unwrap_or_default());
    }
}
