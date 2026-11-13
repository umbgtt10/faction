// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::config::Config;
use faction::members::Members;
use faction::miss_count::MissCount;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::quorum::Quorum;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction::stage::Stage;
use faction::state::State;
use faction::states::removed::Removed;

fn config() -> Config {
    Config::new(0, vec![0, 1, 2], QuorumPolicy::new(2), 1)
}

extern crate alloc;

#[test]
fn cluster_view_shows_what_the_node_held_but_itself_and_its_liveness() {
    // Arrange
    let previous = ClusterViewBuilder::default()
        .with_peer_state(PeerState::Bootstrapped)
        .with_is_pinging_completed(true)
        .with_pinging_peers(vec![0, 1])
        .with_collecting_peers(vec![1, 0])
        .with_required_count(2)
        .with_deadline_missed(true)
        .with_members(Members::new(vec![0, 1, 2]))
        .with_reconfiguration(Reconfiguration::RemovalRequested { peer_id: 0 })
        .with_suspected(vec![2])
        .with_misses(vec![MissCount {
            peer_id: 1,
            misses: 1,
        }])
        .with_last_probed(Some(1))
        .with_quorum(Quorum::Held)
        .build();

    // Act
    let cluster_view = Removed::new(0).cluster_view(&previous);

    // Assert
    assert_eq!(
        cluster_view,
        ClusterViewBuilder::default()
            .with_peer_state(PeerState::Removed)
            .with_is_pinging_completed(true)
            .with_pinging_peers(vec![1])
            .with_collecting_peers(vec![1])
            .with_required_count(2)
            .with_deadline_missed(true)
            .with_members(Members::new(vec![1, 2]))
            .build()
    );
    assert_eq!(cluster_view.peer_state(), PeerState::Removed);
}

#[test]
fn own_removal_answers_the_node_removed_and_leaves_it_removed() {
    // Arrange & Act
    let (outcomes, stage) = Removed::own_removal(0);

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberRemoved { peer_id: 0 }]);
    assert!(matches!(stage, Stage::Removed(_)));
}

#[test]
fn step_of_any_command_is_none() {
    // Arrange
    let removed = Removed::new(0);

    // Act
    let answered = [
        Command::RemovalCommitted { peer_id: 0 },
        Command::ProbeDue,
        Command::JoinApproved { peer_id: 3 },
        Command::Probe,
    ]
    .map(|command| removed.step(command, &config()).is_none());

    // Assert
    assert_eq!(answered, [true; 4]);
}
