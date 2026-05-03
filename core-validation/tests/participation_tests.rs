// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction_core_validation::scenario_harness::ScenarioHarness;

#[test]
fn complete_local_participation_updates_snapshot_and_outputs() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);

    // Act
    let outputs = harness.complete_local_participation(0);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(
        outputs,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    assert!(cluster_view.is_pinging_completed());
    assert_eq!(cluster_view.peer_state(), PeerState::Collecting);
    assert_eq!(cluster_view.collecting_peers().len(), 1);
    assert!(!cluster_view.is_concluded());
}

#[test]
fn apply_participation_accepts_timely_member_observation() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);

    // Act
    let outputs = harness.apply_participation(0, 1);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::ParticipationAccepted { peer_id: 1 }]);
    assert_eq!(cluster_view.pinging_peers().len(), 1);
    assert_eq!(cluster_view.peer_state(), PeerState::Pinging);
    assert!(!cluster_view.is_concluded());
}
