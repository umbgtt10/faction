// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_vibe_observer::NoOpVibeObserver;
use faction::quorum_policy::QuorumPolicy;

use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::vibe::Vibe;
use faction::vibe_config::VibeConfig;
use faction::vibe_input::VibeInput;
use faction::vibe_output::VibeOutput;

fn test_vibe() -> Vibe {
    Vibe::new(
        VibeConfig::new(
            0,
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(4),
            FreshnessPolicy::new(2),
        ),
        Box::new(NoOpVibeObserver),
    )
}

#[test]
fn deal_accepts_participation_observed() {
    let mut vibe = test_vibe();
    let outputs = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let snap = vibe.snapshot();
    assert_eq!(
        outputs,
        vec![VibeOutput::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.phase1_confirmed_count(), 1);
}

#[test]
fn deal_accepts_ready_observed() {
    let mut vibe = test_vibe();
    let outputs = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let snap = vibe.snapshot();
    assert_eq!(outputs, vec![VibeOutput::ReadyAccepted { peer_id: 1 }]);
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.phase2_confirmed_count(), 1);
}

#[test]
fn deal_rejects_local_participation_completed() {
    let mut vibe = test_vibe();
    let outputs = vibe.apply(VibeInput::LocalParticipationCompleted);
    let snap = vibe.snapshot();
    assert!(outputs.is_empty());
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.phase2_confirmed_count(), 0);
}

#[test]
fn deal_rejects_deadline_expired() {
    let mut vibe = test_vibe();
    let outputs = vibe.apply(VibeInput::DeadlineExpired);
    let snap = vibe.snapshot();
    assert!(outputs.is_empty());
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.exit_mode(), None);
}

#[test]
fn stays_in_initial_after_rejected_input() {
    let mut vibe = test_vibe();
    let first = vibe.apply(VibeInput::DeadlineExpired);
    let second = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let snap = vibe.snapshot();
    assert!(first.is_empty());
    assert_eq!(
        second,
        vec![VibeOutput::ParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snap.phase1_confirmed_count(), 1);
}

#[test]
fn multiple_rejected_inputs_keep_initial_unchanged() {
    let mut vibe = test_vibe();
    let r1 = vibe.apply(VibeInput::LocalParticipationCompleted);
    let r2 = vibe.apply(VibeInput::DeadlineExpired);
    let r3 = vibe.apply(VibeInput::LocalParticipationCompleted);
    let snap = vibe.snapshot();
    assert!(r1.is_empty());
    assert!(r2.is_empty());
    assert!(r3.is_empty());
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.phase1_confirmed_count(), 0);
    assert_eq!(snap.phase2_confirmed_count(), 0);
    assert_eq!(snap.exit_mode(), None);
    assert!(!snap.readiness_exited());
    assert!(!snap.local_participation_complete());
}

#[test]
fn punch_participation_non_member_from_initial() {
    let mut vibe = test_vibe();
    let outputs = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    });
    let snap = vibe.snapshot();
    assert_eq!(outputs, vec![VibeOutput::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(snap.phase1_confirmed_count(), 0);
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
}

#[test]
fn punch_participation_delayed_from_initial() {
    let mut vibe = test_vibe();
    let outputs = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 1,
        freshness: 8,
        current_marker: 10,
    });
    let snap = vibe.snapshot();
    assert_eq!(
        outputs,
        vec![VibeOutput::DelayedParticipationAccepted { peer_id: 1 }]
    );
    assert_eq!(snap.phase1_confirmed_count(), 1);
}

#[test]
fn punch_ready_non_member_from_initial() {
    let mut vibe = test_vibe();
    let outputs = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 99,
        freshness: 10,
        current_marker: 10,
    });
    let snap = vibe.snapshot();
    assert_eq!(outputs, vec![VibeOutput::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(snap.phase2_confirmed_count(), 0);
}

#[test]
fn vibe_check_returns_phase1_active_with_zeros() {
    let vibe = test_vibe();
    let snap = vibe.snapshot();
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(snap.exit_mode(), None);
    assert!(!snap.local_participation_complete());
    assert!(!snap.readiness_exited());
    assert_eq!(snap.phase1_confirmed_count(), 0);
    assert_eq!(snap.phase2_confirmed_count(), 0);
    assert_eq!(snap.quorum_threshold(), 4);
}
