// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec;

use faction::conclusion::Conclusion;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction_core_validation::scenario_harness::ScenarioHarness;

#[test]
fn apply_ready_accepts_timely_member_observation_after_local_completion() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);

    // Act
    let outputs = harness.apply_ready(0, 1);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(cluster_view.collecting_peers().len(), 2);
    assert_eq!(cluster_view.peer_state(), PeerState::Collecting);
    assert!(!cluster_view.is_concluded());
}

#[test]
fn apply_ready_reaches_quorum_exit_in_asymmetric_startup_sequence() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1);
    let _ = harness.apply_ready(0, 2);

    // Act
    let outputs = harness.apply_ready(0, 3);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(
        outputs,
        vec![
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::Bootstrapped));
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert!(cluster_view.is_concluded());
    assert_eq!(cluster_view.collecting_peers().len(), 4);
}

#[test]
fn post_exit_ready_is_ignored() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1);
    let _ = harness.apply_ready(0, 2);
    let _ = harness.apply_ready(0, 3);

    // Act
    let outputs = harness.apply_ready(0, 4);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert!(outputs.is_empty());
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::Bootstrapped));
    assert_eq!(cluster_view.collecting_peers().len(), 4);
    assert!(cluster_view.is_concluded());
}

#[test]
fn five_node_asymmetric_startup_reaches_quorum_exit() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(2, 1);
    let _ = harness.complete_local_participation(2);
    let _ = harness.apply_participation(3, 1);
    let _ = harness.complete_local_participation(3);
    let _ = harness.apply_participation(1, 0);
    let _ = harness.complete_local_participation(1);
    let _ = harness.apply_participation(4, 0);
    let _ = harness.complete_local_participation(4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1);
    let _ = harness.apply_ready(0, 2);
    let _ = harness.apply_ready(1, 0);
    let _ = harness.apply_ready(1, 2);

    // Act
    let outputs_0 = harness.apply_ready(0, 3);
    let outputs_1 = harness.apply_ready(1, 3);

    // Assert
    assert_eq!(
        outputs_0,
        vec![
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(
        outputs_1,
        vec![
            Outcome::ReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    let snapshot_0 = harness.cluster_view(0);
    let snapshot_1 = harness.cluster_view(1);
    assert_eq!(snapshot_0.conclusion(), Some(Conclusion::Bootstrapped));
    assert_eq!(snapshot_1.conclusion(), Some(Conclusion::Bootstrapped));
    assert!(snapshot_0.is_concluded());
    assert!(snapshot_1.is_concluded());
    assert_eq!(snapshot_0.peer_state(), PeerState::Bootstrapped);
    assert_eq!(snapshot_1.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn early_ready_signals_accumulate_before_local_participation_completion() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let outputs_peer_1 = harness.apply_ready(0, 1);
    let outputs_peer_2 = harness.apply_ready(0, 2);
    let outputs_peer_3 = harness.apply_ready(0, 3);
    let intermediate_snapshot = harness.cluster_view(0);

    // Act
    let outputs = harness.complete_local_participation(0);

    // Assert
    assert_eq!(outputs_peer_1, vec![Outcome::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(outputs_peer_2, vec![Outcome::ReadyAccepted { peer_id: 2 }]);
    assert_eq!(outputs_peer_3, vec![Outcome::ReadyAccepted { peer_id: 3 }]);
    assert_eq!(intermediate_snapshot.collecting_peers().len(), 3);
    assert_eq!(intermediate_snapshot.peer_state(), PeerState::Pinging);
    assert!(!intermediate_snapshot.is_pinging_completed());
    assert!(!intermediate_snapshot.is_concluded());
    assert_eq!(
        outputs,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    let cluster_view = harness.cluster_view(0);
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::Bootstrapped));
    assert!(cluster_view.is_concluded());
    assert_eq!(cluster_view.collecting_peers().len(), 4);
}
