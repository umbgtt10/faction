// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction_core_validation::scenario_harness::ScenarioHarness;

#[test]
fn duplicate_signals_across_nodes_remain_idempotent() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1);

    // Act
    let outputs = harness.apply_ready(0, 1);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::DuplicateReadyIgnored { peer_id: 1 }]);
    assert_eq!(cluster_view.collecting_peers().len(), 2);
    assert_eq!(cluster_view.peer_state(), PeerState::Collecting);
    assert!(!cluster_view.is_concluded());
}

#[test]
fn non_member_signal_does_not_perturb_multi_peer_state() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1);

    // Act
    let outputs = harness.apply_ready(0, 99);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(cluster_view.collecting_peers().len(), 2);
    assert_eq!(cluster_view.peer_state(), PeerState::Collecting);
    assert!(!cluster_view.is_concluded());
}
