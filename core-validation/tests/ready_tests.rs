// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec;

use faction::conclusion::Conclusion;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction_core_validation::scenario_harness::ScenarioHarness;

#[test]
fn apply_ready_accepts_timely_member_observation_after_local_completion() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);

    // Act
    let outputs = harness.apply_ready(0, 1, 10);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(cluster_view.collecting_peers().len(), 2);
    assert_eq!(cluster_view.peer_state(), PeerState::Collecting);
    assert!(!cluster_view.is_exited());
}

#[test]
fn apply_ready_accepts_delayed_member_observation_within_margin() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);

    // Act
    let outputs = harness.apply_ready(0, 1, 8);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::DelayedReadyAccepted { peer_id: 1 }]);
    assert_eq!(cluster_view.collecting_peers().len(), 2);
    assert!(!cluster_view.is_exited());
}

#[test]
fn apply_ready_rejects_stale_member_observation() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);

    // Act
    let outputs = harness.apply_ready(0, 1, 7);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(outputs, vec![Outcome::StaleReadyIgnored { peer_id: 1 }]);
    assert_eq!(cluster_view.collecting_peers().len(), 1);
    assert_eq!(cluster_view.peer_state(), PeerState::Collecting);
    assert!(!cluster_view.is_exited());
}

#[test]
fn apply_ready_reaches_quorum_exit_in_asymmetric_startup_sequence() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);
    let _ = harness.apply_ready(0, 2, 10);

    // Act
    let outputs = harness.apply_ready(0, 3, 10);
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
    assert_eq!(cluster_view.exit_mode(), Some(Conclusion::Bootstrapped));
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert!(cluster_view.is_exited());
    assert_eq!(cluster_view.collecting_peers().len(), 4);
}

#[test]
fn delayed_signals_within_margin_still_allow_quorum_exit() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 8);
    let _ = harness.apply_ready(0, 2, 9);

    // Act
    let outputs = harness.apply_ready(0, 3, 8);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(
        outputs,
        vec![
            Outcome::DelayedReadyAccepted { peer_id: 3 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ]
    );
    assert_eq!(cluster_view.exit_mode(), Some(Conclusion::Bootstrapped));
    assert_eq!(cluster_view.collecting_peers().len(), 4);
    assert!(cluster_view.is_exited());
}

#[test]
fn post_exit_ready_is_ignored() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);
    let _ = harness.apply_ready(0, 2, 10);
    let _ = harness.apply_ready(0, 3, 10);

    // Act
    let outputs = harness.apply_ready(0, 4, 10);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert!(outputs.is_empty());
    assert_eq!(cluster_view.exit_mode(), Some(Conclusion::Bootstrapped));
    assert_eq!(cluster_view.collecting_peers().len(), 4);
    assert!(cluster_view.is_exited());
}

#[test]
fn five_node_asymmetric_startup_reaches_quorum_exit() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(5);
    let _ = harness.complete_local_participation(2);
    let _ = harness.complete_local_participation(3);
    harness.advance_to(8);
    let _ = harness.complete_local_participation(1);
    let _ = harness.complete_local_participation(4);
    harness.advance_to(10);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1, 10);
    let _ = harness.apply_ready(0, 2, 10);
    let _ = harness.apply_ready(1, 0, 10);
    let _ = harness.apply_ready(1, 2, 10);

    // Act
    let outputs_0 = harness.apply_ready(0, 3, 10);
    let outputs_1 = harness.apply_ready(1, 3, 10);

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
    assert_eq!(snapshot_0.exit_mode(), Some(Conclusion::Bootstrapped));
    assert_eq!(snapshot_1.exit_mode(), Some(Conclusion::Bootstrapped));
    assert!(snapshot_0.is_exited());
    assert!(snapshot_1.is_exited());
    assert_eq!(snapshot_0.peer_state(), PeerState::Bootstrapped);
    assert_eq!(snapshot_1.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn early_ready_signals_accumulate_before_local_participation_completion() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4, 2);
    harness.advance_to(10);
    let outputs_peer_1 = harness.apply_ready(0, 1, 10);
    let outputs_peer_2 = harness.apply_ready(0, 2, 10);
    let outputs_peer_3 = harness.apply_ready(0, 3, 10);
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
    assert!(!intermediate_snapshot.is_exited());
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
    assert_eq!(cluster_view.exit_mode(), Some(Conclusion::Bootstrapped));
    assert!(cluster_view.is_exited());
    assert_eq!(cluster_view.collecting_peers().len(), 4);
}
