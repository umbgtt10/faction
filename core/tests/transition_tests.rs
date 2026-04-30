// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::exit_mode::ExitMode;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::transition::Transition;

fn cluster_view(pinging_peers: Vec<u64>, collecting_peers: Vec<u64>) -> ClusterView {
    ClusterView::new(
        PeerState::Pinging,
        false,
        pinging_peers,
        collecting_peers,
        4,
    )
}

fn snapshot_exited() -> ClusterView {
    ClusterView::new(
        PeerState::Bootstrapped,
        true,
        vec![1, 2, 3],
        vec![1, 2, 3, 4, 5],
        4,
    )
}

#[test]
fn new_stores_previous_state() {
    // Arrange
    let prev = cluster_view(vec![1], vec![]);
    let next = cluster_view(vec![1], vec![1]);
    let outputs = vec![Outcome::ParticipationAccepted { peer_id: 1 }];

    // Act
    let transition = Transition::new(prev, outputs, next);

    // Assert
    assert_eq!(transition.previous_view(), cluster_view(vec![1], vec![]));
}

#[test]
fn new_stores_new_state() {
    // Arrange
    let prev = cluster_view(vec![1], vec![]);
    let next = cluster_view(vec![1], vec![1]);
    let outputs = vec![Outcome::ParticipationAccepted { peer_id: 1 }];

    // Act
    let transition = Transition::new(prev, outputs, next);

    // Assert
    assert_eq!(transition.new_view(), cluster_view(vec![1], vec![1]));
}

#[test]
fn new_stores_outputs() {
    // Arrange
    let prev = cluster_view(vec![], vec![]);
    let next = cluster_view(vec![1], vec![]);
    let outputs = vec![
        Outcome::LocalParticipationCompleted,
        Outcome::BroadcastLocalReady,
    ];

    // Act
    let transition = Transition::new(prev, outputs.clone(), next);

    // Assert
    assert_eq!(transition.outputs(), &outputs);
}

#[test]
fn new_handles_empty_outputs() {
    // Arrange
    let prev = cluster_view(vec![], vec![]);
    let next = cluster_view(vec![], vec![1]);
    let outputs = vec![];

    // Act
    let transition = Transition::new(prev, outputs.clone(), next);

    // Assert
    assert!(transition.outputs().is_empty());
}

#[test]
fn previous_state_preserves_full_snapshot() {
    // Arrange
    let prev = snapshot_exited();
    let next = cluster_view(vec![], vec![]);
    let outputs = vec![];

    // Act
    let transition = Transition::new(prev, outputs, next);

    // Assert
    let result = transition.previous_view();
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.exit_mode(), Some(ExitMode::Bootstrapped));
    assert!(result.is_pinging_completed());
    assert!(result.is_exited());
    assert_eq!(result.pinging_peers().len(), 3);
    assert_eq!(result.collecting_peers().len(), 5);
}

#[test]
fn new_state_preserves_full_snapshot() {
    // Arrange
    let prev = cluster_view(vec![], vec![]);
    let next = snapshot_exited();

    // Act
    let transition = Transition::new(prev, vec![], next);

    // Assert
    let result = transition.new_view();
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.exit_mode(), Some(ExitMode::Bootstrapped));
    assert!(result.is_pinging_completed());
    assert!(result.is_exited());
    assert_eq!(result.pinging_peers().len(), 3);
    assert_eq!(result.collecting_peers().len(), 5);
}

#[test]
fn outputs_are_immutable() {
    // Arrange
    let prev = cluster_view(vec![], vec![]);
    let next = cluster_view(vec![1], vec![]);
    let outputs = vec![Outcome::ReadyAccepted { peer_id: 1 }];

    // Act
    let transition = Transition::new(prev, outputs, next);

    // Assert
    assert_eq!(transition.outputs().len(), 1);
    assert_eq!(
        transition.outputs()[0],
        Outcome::ReadyAccepted { peer_id: 1 }
    );
}

#[test]
fn clone_produces_equal_transition() {
    // Arrange
    let prev = cluster_view(vec![1], vec![]);
    let next = cluster_view(vec![1], vec![1, 2]);
    let outputs = vec![
        Outcome::ReadyAccepted { peer_id: 3 },
        Outcome::ReadyQuorumReached,
        Outcome::Exited {
            mode: ExitMode::Bootstrapped,
        },
    ];
    let transition = Transition::new(prev, outputs.clone(), next);

    // Act
    let cloned = transition.clone();

    // Assert
    assert_eq!(cloned.previous_view(), transition.previous_view());
    assert_eq!(cloned.new_view(), transition.new_view());
    assert_eq!(cloned.outputs(), transition.outputs());
}

#[test]
fn debug_format_does_not_panic() {
    // Arrange
    let transition = Transition::new(
        cluster_view(vec![], vec![]),
        vec![],
        cluster_view(vec![1], vec![1]),
    );

    // Act & Assert
    let _ = format!("{:?}", transition);
}
