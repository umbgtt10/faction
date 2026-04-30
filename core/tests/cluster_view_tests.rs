// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::node_state::NodeState;
use faction::readiness_exit_mode::ReadinessExitMode;

fn base() -> ClusterView {
    ClusterView::new(
        NodeState::Bootstrapped,
        true,
        vec![1, 2, 3, 4, 5],
        vec![1, 2, 3, 4, 5, 6, 7],
        3,
    )
}

#[test]
fn with_node_state_updates_only_node_state() {
    // Arrange & Act
    let result = base().with_node_state(NodeState::Pinging);

    // Assert
    assert_eq!(result.node_state(), NodeState::Pinging);
    assert_eq!(result.exit_mode(), None);
    assert!(result.is_pinging_completed());
    assert!(!result.readiness_exited());
    assert_eq!(result.pinging_peers(), &[1, 2, 3, 4, 5]);
    assert_eq!(result.collecting_peers(), &[1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(result.required_count(), 3);
}

#[test]
fn with_collecting_peers_updates_only_collecting_peers() {
    // Arrange & Act
    let result = base().with_collecting_peers(vec![99]);

    // Assert
    assert_eq!(result.node_state(), NodeState::Bootstrapped);
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Bootstrapped));
    assert!(result.is_pinging_completed());
    assert!(result.readiness_exited());
    assert_eq!(result.pinging_peers(), &[1, 2, 3, 4, 5]);
    assert_eq!(result.collecting_peers(), &[99]);
    assert_eq!(result.required_count(), 3);
}
