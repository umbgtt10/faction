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

fn vibe() -> Vibe {
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

fn reach_deadline_from_phase1() -> Vibe {
    let mut m = vibe();
    let _ = m.apply(VibeInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = m.apply(VibeInput::DeadlineExpired);
    m
}

fn reach_deadline_from_phase2() -> Vibe {
    let mut m = vibe();
    let _ = m.apply(VibeInput::ParticipationObserved {
        peer_id: 1,
        freshness: 10,
        current_marker: 10,
    });
    let _ = m.apply(VibeInput::LocalParticipationCompleted);
    let _ = m.apply(VibeInput::DeadlineExpired);
    m
}

#[test]
fn deal_rejects_participation_observed() {
    let mut m = reach_deadline_from_phase1();
    let snap_before = m.snapshot();

    let outputs = m.apply(VibeInput::ParticipationObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });

    assert!(outputs.is_empty());
    assert_eq!(m.snapshot(), snap_before);
}

#[test]
fn deal_rejects_ready_observed() {
    let mut m = reach_deadline_from_phase1();
    let snap_before = m.snapshot();

    let outputs = m.apply(VibeInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });

    assert!(outputs.is_empty());
    assert_eq!(m.snapshot(), snap_before);
}

#[test]
fn deal_rejects_local_participation_completed() {
    let mut m = reach_deadline_from_phase1();
    let snap_before = m.snapshot();

    let outputs = m.apply(VibeInput::LocalParticipationCompleted);

    assert!(outputs.is_empty());
    assert_eq!(m.snapshot(), snap_before);
}

#[test]
fn deal_rejects_deadline_expired() {
    let mut m = reach_deadline_from_phase1();
    let snap_before = m.snapshot();

    let outputs = m.apply(VibeInput::DeadlineExpired);

    assert!(outputs.is_empty());
    assert_eq!(m.snapshot(), snap_before);
}

#[test]
fn vibe_check_after_deadline_from_phase1() {
    let m = reach_deadline_from_phase1();
    let s = m.snapshot();

    assert_eq!(
        s.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(s.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(s.readiness_exited());
    assert!(!s.local_participation_complete());
    assert_eq!(s.phase1_confirmed_count(), 1);
    assert_eq!(s.phase2_confirmed_count(), 0);
}

#[test]
fn vibe_check_after_deadline_from_phase2() {
    let m = reach_deadline_from_phase2();
    let s = m.snapshot();

    assert_eq!(
        s.lifecycle_state(),
        ReadinessLifecycleState::ReadyByDeadline
    );
    assert_eq!(s.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(s.readiness_exited());
    assert!(s.local_participation_complete());
    assert_eq!(s.phase1_confirmed_count(), 1);
    assert_eq!(s.phase2_confirmed_count(), 1);
}

#[test]
fn post_deadline_inputs_leave_state_unchanged() {
    let mut m = reach_deadline_from_phase1();
    let snapshot_before = m.snapshot();

    let _ = m.apply(VibeInput::ParticipationObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = m.apply(VibeInput::ReadyObserved {
        peer_id: 2,
        freshness: 10,
        current_marker: 10,
    });
    let _ = m.apply(VibeInput::LocalParticipationCompleted);
    let _ = m.apply(VibeInput::DeadlineExpired);

    assert_eq!(m.snapshot(), snapshot_before);
}
