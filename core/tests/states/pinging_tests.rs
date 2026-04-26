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
use faction::vibe_output::VibeOutput;

const PEER_SET: &[u64] = &[0, 1, 2, 3, 4];
const THRESHOLD: usize = 4;
const MAX_DELAY: u64 = 2;
const MARKER: u64 = 10;
const TIMELY: u64 = 10;
const DELAYED: u64 = 8;
const STALE: u64 = 7;

fn vibe_in_phase1() -> Vibe {
    let mut vibe = Vibe::new(
        VibeConfig::new(
            0,
            PEER_SET.to_vec(),
            QuorumPolicy::new(THRESHOLD),
            FreshnessPolicy::new(MAX_DELAY),
        ),
        Box::new(NoOpVibeObserver),
    );
    let _ = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    vibe
}

fn p1(vibe: &Vibe) -> usize {
    vibe.snapshot().phase1_confirmed_count()
}
fn p2(vibe: &Vibe) -> usize {
    vibe.snapshot().phase2_confirmed_count()
}

#[test]
fn deal_accepts_participation_observed() {
    // Arrange
    let mut vibe = vibe_in_phase1();

    // Act
    let result = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });

    // Assert
    assert_eq!(
        result,
        vec![VibeOutput::ParticipationAccepted { peer_id: 2 }]
    );
}

#[test]
fn deal_accepts_ready_observed() {
    let mut vibe = vibe_in_phase1();
    let result = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![VibeOutput::ReadyAccepted { peer_id: 2 }]);
}

#[test]
fn deal_accepts_local_participation_completed() {
    let mut vibe = vibe_in_phase1();
    let result = vibe.apply(VibeInput::LocalParticipationCompleted);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], VibeOutput::LocalParticipationCompleted);
    assert_eq!(result[1], VibeOutput::BroadcastLocalReady);
}

#[test]
fn deal_accepts_deadline_expired() {
    let mut vibe = vibe_in_phase1();
    let result = vibe.apply(VibeInput::DeadlineExpired);
    assert_eq!(
        result,
        vec![VibeOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline
        }]
    );
}

#[test]
fn participation_observed_non_member() {
    let mut vibe = vibe_in_phase1();
    let before = p1(&vibe);
    let result = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 99,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![VibeOutput::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(p1(&vibe), before);
}

#[test]
fn participation_observed_stale() {
    let mut vibe = vibe_in_phase1();
    let before = p1(&vibe);
    let result = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 2,
        freshness: STALE,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![VibeOutput::StaleParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(p1(&vibe), before);
}

#[test]
fn participation_observed_duplicate() {
    let mut vibe = vibe_in_phase1();
    let _ = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let before = p1(&vibe);
    let result = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![VibeOutput::DuplicateParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(p1(&vibe), before);
}

#[test]
fn participation_observed_first_timely() {
    let mut vibe = vibe_in_phase1();
    let before = p1(&vibe);
    let result = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![VibeOutput::ParticipationAccepted { peer_id: 3 }]
    );
    assert_eq!(p1(&vibe), before + 1);
}

#[test]
fn participation_observed_first_delayed() {
    let mut vibe = vibe_in_phase1();
    let before = p1(&vibe);
    let result = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 3,
        freshness: DELAYED,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![VibeOutput::DelayedParticipationAccepted { peer_id: 3 }]
    );
    assert_eq!(p1(&vibe), before + 1);
}

#[test]
fn ready_observed_non_member() {
    let mut vibe = vibe_in_phase1();
    let before = p2(&vibe);
    let result = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 99,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![VibeOutput::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(p2(&vibe), before);
}

#[test]
fn ready_observed_stale() {
    let mut vibe = vibe_in_phase1();
    let before = p2(&vibe);
    let result = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 2,
        freshness: STALE,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![VibeOutput::StaleReadyIgnored { peer_id: 2 }]);
    assert_eq!(p2(&vibe), before);
}

#[test]
fn ready_observed_duplicate() {
    let mut vibe = vibe_in_phase1();
    let _ = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let before = p2(&vibe);
    let result = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![VibeOutput::DuplicateReadyIgnored { peer_id: 2 }]
    );
    assert_eq!(p2(&vibe), before);
}

#[test]
fn ready_observed_first_timely() {
    let mut vibe = vibe_in_phase1();
    let before = p2(&vibe);
    let result = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![VibeOutput::ReadyAccepted { peer_id: 3 }]);
    assert_eq!(p2(&vibe), before + 1);
}

#[test]
fn ready_observed_first_delayed() {
    let mut vibe = vibe_in_phase1();
    let before = p2(&vibe);
    let result = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 3,
        freshness: DELAYED,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![VibeOutput::DelayedReadyAccepted { peer_id: 3 }]
    );
    assert_eq!(p2(&vibe), before + 1);
}

#[test]
fn local_completion_no_quorum() {
    let mut vibe = vibe_in_phase1();
    let result = vibe.apply(VibeInput::LocalParticipationCompleted);
    // Arrange & Act
    assert_eq!(
        result,
        vec![
            VibeOutput::LocalParticipationCompleted,
            VibeOutput::BroadcastLocalReady,
        ]
    );
    // Assert
    let snap = vibe.snapshot();
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase2Active
    );
    assert!(snap.local_participation_complete());
    assert!(!snap.readiness_exited());
}

#[test]
fn local_completion_triggers_quorum() {
    // Arrange
    let mut vibe = Vibe::new(
        VibeConfig::new(
            0,
            PEER_SET.to_vec(),
            QuorumPolicy::new(4),
            FreshnessPolicy::new(MAX_DELAY),
        ),
        Box::new(NoOpVibeObserver),
    );
    let _ = vibe.apply(VibeInput::ParticipationObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = vibe.apply(VibeInput::ReadyObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });

    // Act
    let result = vibe.apply(VibeInput::LocalParticipationCompleted);

    // Assert
    assert_eq!(
        result,
        vec![
            VibeOutput::LocalParticipationCompleted,
            VibeOutput::BroadcastLocalReady,
            VibeOutput::ReadyQuorumReached,
            VibeOutput::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    let snap = vibe.snapshot();
    assert!(snap.readiness_exited());
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Quorum));
}

#[test]
fn deadline_expired_in_phase1() {
    let mut vibe = vibe_in_phase1();
    // Act & Assert
    let result = vibe.apply(VibeInput::DeadlineExpired);
    assert_eq!(
        result,
        vec![VibeOutput::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    );
    let snap = vibe.snapshot();
    assert!(snap.readiness_exited());
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Deadline));
}

#[test]
fn vibe_check_in_phase1() {
    // Arrange & Act
    let vibe = vibe_in_phase1();
    let snap = vibe.snapshot();

    // Assert
    assert_eq!(
        snap.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert!(!snap.readiness_exited());
    assert!(!snap.local_participation_complete());
    assert_eq!(snap.exit_mode(), None);
    assert_eq!(snap.phase1_confirmed_count(), 1);
    assert_eq!(snap.phase2_confirmed_count(), 0);
    assert_eq!(snap.quorum_threshold(), THRESHOLD);
}
