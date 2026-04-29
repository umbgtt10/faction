// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::readiness_lifecycle_state::ReadinessLifecycleState;
use faction::snapshot::Snapshot;
use faction::state_snapshot::StateSnapshot;
use faction::states::pinging::Pinging;

const PEER_SET: &[u64] = &[0, 1, 2, 3, 4];
const THRESHOLD: usize = 4;
const MAX_DELAY: u64 = 2;
const MARKER: u64 = 10;
const TIMELY: u64 = 10;
const DELAYED: u64 = 8;
const STALE: u64 = 7;

fn machine_in_phase1() -> Faction {
    let mut faction = Faction::new(
        Config::new(
            0,
            PEER_SET.to_vec(),
            QuorumPolicy::new(THRESHOLD),
            FreshnessPolicy::new(MAX_DELAY),
        ),
        Box::new(NoOpObserver),
    );
    let _ = faction.apply(Command::ParticipationObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    faction
}

fn p1(faction: &Faction) -> usize {
    faction.snapshot().phase1_confirmed_count()
}
fn p2(faction: &Faction) -> usize {
    faction.snapshot().phase2_confirmed_count()
}

#[test]
fn deal_accepts_participation_observed() {
    // Arrange
    let mut faction = machine_in_phase1();

    // Act
    let result = faction.apply(Command::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });

    // Assert
    assert_eq!(result, vec![Outcome::ParticipationAccepted { peer_id: 2 }]);
}

#[test]
fn deal_accepts_ready_observed() {
    let mut faction = machine_in_phase1();
    let result = faction.apply(Command::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![Outcome::ReadyAccepted { peer_id: 2 }]);
}

#[test]
fn deal_accepts_local_participation_completed() {
    let mut faction = machine_in_phase1();
    let result = faction.apply(Command::LocalParticipationCompleted);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], Outcome::LocalParticipationCompleted);
    assert_eq!(result[1], Outcome::BroadcastLocalReady);
}

#[test]
fn deal_accepts_deadline_expired() {
    let mut faction = machine_in_phase1();
    let result = faction.apply(Command::DeadlineExpired);
    assert_eq!(
        result,
        vec![Outcome::ReadinessExited {
            mode: ReadinessExitMode::Deadline
        }]
    );
}

#[test]
fn participation_observed_non_member() {
    let mut faction = machine_in_phase1();
    let before = p1(&faction);
    let result = faction.apply(Command::ParticipationObserved {
        peer_id: 99,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(p1(&faction), before);
}

#[test]
fn participation_observed_stale() {
    let mut faction = machine_in_phase1();
    let before = p1(&faction);
    let result = faction.apply(Command::ParticipationObserved {
        peer_id: 2,
        freshness: STALE,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![Outcome::StaleParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(p1(&faction), before);
}

#[test]
fn participation_observed_duplicate() {
    let mut faction = machine_in_phase1();
    let _ = faction.apply(Command::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let before = p1(&faction);
    let result = faction.apply(Command::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![Outcome::DuplicateParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(p1(&faction), before);
}

#[test]
fn participation_observed_first_timely() {
    let mut faction = machine_in_phase1();
    let before = p1(&faction);
    let result = faction.apply(Command::ParticipationObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![Outcome::ParticipationAccepted { peer_id: 3 }]);
    assert_eq!(p1(&faction), before + 1);
}

#[test]
fn participation_observed_first_delayed() {
    let mut faction = machine_in_phase1();
    let before = p1(&faction);
    let result = faction.apply(Command::ParticipationObserved {
        peer_id: 3,
        freshness: DELAYED,
        current_marker: MARKER,
    });
    assert_eq!(
        result,
        vec![Outcome::DelayedParticipationAccepted { peer_id: 3 }]
    );
    assert_eq!(p1(&faction), before + 1);
}

#[test]
fn ready_observed_non_member() {
    let mut faction = machine_in_phase1();
    let before = p2(&faction);
    let result = faction.apply(Command::ReadyObserved {
        peer_id: 99,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(p2(&faction), before);
}

#[test]
fn ready_observed_stale() {
    let mut faction = machine_in_phase1();
    let before = p2(&faction);
    let result = faction.apply(Command::ReadyObserved {
        peer_id: 2,
        freshness: STALE,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![Outcome::StaleReadyIgnored { peer_id: 2 }]);
    assert_eq!(p2(&faction), before);
}

#[test]
fn ready_observed_duplicate() {
    let mut faction = machine_in_phase1();
    let _ = faction.apply(Command::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let before = p2(&faction);
    let result = faction.apply(Command::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![Outcome::DuplicateReadyIgnored { peer_id: 2 }]);
    assert_eq!(p2(&faction), before);
}

#[test]
fn ready_observed_first_timely() {
    let mut faction = machine_in_phase1();
    let before = p2(&faction);
    let result = faction.apply(Command::ReadyObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![Outcome::ReadyAccepted { peer_id: 3 }]);
    assert_eq!(p2(&faction), before + 1);
}

#[test]
fn ready_observed_first_delayed() {
    let mut faction = machine_in_phase1();
    let before = p2(&faction);
    let result = faction.apply(Command::ReadyObserved {
        peer_id: 3,
        freshness: DELAYED,
        current_marker: MARKER,
    });
    assert_eq!(result, vec![Outcome::DelayedReadyAccepted { peer_id: 3 }]);
    assert_eq!(p2(&faction), before + 1);
}

#[test]
fn local_completion_no_quorum() {
    let mut faction = machine_in_phase1();
    let result = faction.apply(Command::LocalParticipationCompleted);
    // Arrange & Act
    assert_eq!(
        result,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    // Assert
    let snap = faction.snapshot();
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
    let mut faction = Faction::new(
        Config::new(
            0,
            PEER_SET.to_vec(),
            QuorumPolicy::new(4),
            FreshnessPolicy::new(MAX_DELAY),
        ),
        Box::new(NoOpObserver),
    );
    let _ = faction.apply(Command::ParticipationObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = faction.apply(Command::ReadyObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = faction.apply(Command::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = faction.apply(Command::ReadyObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });

    // Act
    let result = faction.apply(Command::LocalParticipationCompleted);

    // Assert
    assert_eq!(
        result,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::ReadyQuorumReached,
            Outcome::ReadinessExited {
                mode: ReadinessExitMode::Quorum,
            },
        ]
    );
    let snap = faction.snapshot();
    assert!(snap.readiness_exited());
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Quorum));
}

#[test]
fn deadline_expired_in_phase1() {
    let mut faction = machine_in_phase1();
    // Act & Assert
    let result = faction.apply(Command::DeadlineExpired);
    assert_eq!(
        result,
        vec![Outcome::ReadinessExited {
            mode: ReadinessExitMode::Deadline,
        }]
    );
    let snap = faction.snapshot();
    assert!(snap.readiness_exited());
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Deadline));
}

#[test]
fn vibe_check_in_phase1() {
    // Arrange & Act
    let faction = machine_in_phase1();
    let snap = faction.snapshot();

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

#[test]
fn pinging_state_snapshot_inherits_correctly() {
    // Arrange
    let pinging = Pinging::new(5);
    let prev = Snapshot::new(
        ReadinessLifecycleState::Phase2Active,
        Some(ReadinessExitMode::Deadline),
        true,
        true,
        99,
        99,
        4,
    );

    // Act
    let result = pinging.state_snapshot(&prev);

    // Assert
    assert_eq!(
        result.lifecycle_state(),
        ReadinessLifecycleState::Phase1Active
    );
    assert_eq!(result.phase1_confirmed_count(), 0);
    assert_eq!(result.phase2_confirmed_count(), 0);
    assert_eq!(result.exit_mode(), Some(ReadinessExitMode::Deadline));
    assert!(result.local_participation_complete());
    assert!(result.readiness_exited());
}
