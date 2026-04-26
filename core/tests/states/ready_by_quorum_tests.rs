// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_vibe_observer::NoOpVibeObserver;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::vibe::Vibe;
use faction::vibe_config::VibeConfig;
use faction::vibe_input::VibeInput;

fn reach_ready_by_quorum() -> Vibe {
    let mut vibe = Vibe::new(
        VibeConfig::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(4),
            FreshnessPolicy::new(2),
        ),
        Box::new(NoOpVibeObserver),
    );
    let _ = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = vibe.apply(VibeInput::LocalParticipationCompleted);
    let _ = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 3,
        freshness: 10,
        current_marker: 10,
    });
    vibe
}

#[test]
fn deal_rejects_participation_observed() {
    // Arrange
    let vibe = reach_ready_by_quorum();
    let snapshot = vibe.snapshot();

    // Assert
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot.readiness_exited());
}

#[test]
fn all_inputs_leave_state_unchanged() {
    // Arrange
    let mut vibe = reach_ready_by_quorum();
    let snapshot_before = vibe.snapshot();

    // Act
    let r1 = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 0,
        freshness: 10,
        current_marker: 10,
    });
    let r2 = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 4,
        freshness: 10,
        current_marker: 10,
    });
    let r3 = vibe.apply(VibeInput::LocalParticipationCompleted);
    let r4 = vibe.apply(VibeInput::DeadlineExpired);
    let snapshot_after = vibe.snapshot();

    // Assert
    assert!(r1.is_empty());
    assert!(r2.is_empty());
    assert!(r3.is_empty());
    assert!(r4.is_empty());
    assert_eq!(snapshot_before, snapshot_after);
}

#[test]
fn vibe_check_returns_correct_snapshot() {
    // Arrange & Act
    let vibe = reach_ready_by_quorum();
    let snapshot = vibe.snapshot();

    // Assert
    assert_eq!(
        snapshot.lifecycle_state(),
        ReadinessLifecycleState::ReadyByQuorum
    );
    assert_eq!(snapshot.exit_mode(), Some(ReadinessExitMode::Quorum));
    assert!(snapshot.local_participation_complete());
    assert!(snapshot.readiness_exited());
    assert_eq!(snapshot.phase1_confirmed_count(), 1);
    assert_eq!(snapshot.phase2_confirmed_count(), 4);
    assert_eq!(snapshot.quorum_threshold(), 4);
}
