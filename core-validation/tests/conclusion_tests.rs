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
fn slow_member_does_not_block_quorum_exit() {
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
    assert_eq!(cluster_view.collecting_peers().len(), 4);
    assert!(cluster_view.is_concluded());
}

#[test]
fn expire_deadline_exits_by_deadline() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);

    // Act
    let outputs = harness.expire_deadline(0);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(
        outputs,
        vec![Outcome::Concluded {
            mode: Conclusion::TimedOut,
        }]
    );
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::TimedOut));
    assert_eq!(cluster_view.peer_state(), PeerState::TimedOut);
    assert!(cluster_view.is_concluded());
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
fn repeated_deadline_expiry_remains_idempotent() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);
    let _ = harness.expire_deadline(0);

    // Act
    let outputs = harness.expire_deadline(0);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert!(outputs.is_empty());
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::TimedOut));
    assert_eq!(cluster_view.peer_state(), PeerState::TimedOut);
    assert!(cluster_view.is_concluded());
}

#[test]
fn deadline_fallback_preserves_progress_when_quorum_never_forms() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);
    let _ = harness.complete_local_participation(1);
    let _ = harness.complete_local_participation(2);
    let _ = harness.complete_local_participation(3);
    let _ = harness.complete_local_participation(4);
    let _ = harness.apply_ready(0, 1);
    let _ = harness.apply_ready(0, 2);

    // Act
    let outputs = harness.expire_deadline(0);

    // Assert
    assert_eq!(
        outputs,
        vec![Outcome::Concluded {
            mode: Conclusion::TimedOut,
        }]
    );
    let cluster_view = harness.cluster_view(0);
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::TimedOut));
    assert_eq!(cluster_view.peer_state(), PeerState::TimedOut);
    assert!(cluster_view.is_concluded());
    assert_eq!(cluster_view.collecting_peers().len(), 3);
}

#[test]
fn post_exit_ready_signals_are_harmless_across_multiple_nodes() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    for i in 0..5 {
        let _ = harness.apply_participation(i, 1);
        let _ = harness.complete_local_participation(i);
    }
    let _ = harness.apply_ready(0, 1);
    let _ = harness.apply_ready(0, 2);
    let _ = harness.apply_ready(0, 3);
    let _ = harness.apply_ready(1, 0);
    let _ = harness.apply_ready(1, 2);
    let _ = harness.apply_ready(1, 3);
    let _ = harness.apply_ready(2, 0);
    let _ = harness.apply_ready(2, 1);
    let _ = harness.apply_ready(2, 3);

    // Act
    let outputs_0 = harness.apply_ready(0, 4);
    let outputs_1 = harness.apply_ready(1, 4);
    let outputs_2 = harness.apply_ready(2, 4);

    // Assert
    assert!(outputs_0.is_empty());
    assert!(outputs_1.is_empty());
    assert!(outputs_2.is_empty());
    let snapshot_0 = harness.cluster_view(0);
    let snapshot_1 = harness.cluster_view(1);
    let snapshot_2 = harness.cluster_view(2);
    assert_eq!(snapshot_0.conclusion(), Some(Conclusion::Bootstrapped));
    assert!(snapshot_0.is_concluded());
    assert_eq!(snapshot_1.conclusion(), Some(Conclusion::Bootstrapped));
    assert!(snapshot_1.is_concluded());
    assert_eq!(snapshot_2.conclusion(), Some(Conclusion::Bootstrapped));
    assert!(snapshot_2.is_concluded());
}

#[test]
fn deadline_from_pinging() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);

    // Act
    let outputs = harness.expire_deadline(0);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert_eq!(
        outputs,
        vec![Outcome::Concluded {
            mode: Conclusion::TimedOut,
        }]
    );
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::TimedOut));
    assert_eq!(cluster_view.peer_state(), PeerState::TimedOut);
    assert!(cluster_view.is_concluded());
    assert!(!cluster_view.is_pinging_completed());
    assert_eq!(cluster_view.pinging_peers().len(), 1);
    assert_eq!(cluster_view.collecting_peers().len(), 0);
}

#[test]
fn deadline_from_bootstrapped_is_noop() {
    // Arrange
    let mut harness = ScenarioHarness::new(vec![0, 1, 2, 3, 4], 4);
    let _ = harness.apply_participation(0, 1);
    let _ = harness.complete_local_participation(0);
    let _ = harness.apply_ready(0, 1);
    let _ = harness.apply_ready(0, 2);
    let _ = harness.apply_ready(0, 3);

    // Act
    let outputs = harness.expire_deadline(0);
    let cluster_view = harness.cluster_view(0);

    // Assert
    assert!(outputs.is_empty());
    assert_eq!(cluster_view.conclusion(), Some(Conclusion::Bootstrapped));
    assert_eq!(cluster_view.collecting_peers().len(), 4);
    assert!(cluster_view.is_concluded());
}
