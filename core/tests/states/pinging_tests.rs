// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::readiness_exit_mode::ReadinessExitMode;
use faction::node_state::NodeState;
use faction::state::State;

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
    let _ = faction.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    faction
}

fn p1(faction: &mut Faction) -> usize {
    match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view.phase1_confirmed_count(),
        _ => unreachable!(),
    }
}
fn p2(faction: &mut Faction) -> usize {
    match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view.phase2_confirmed_count(),
        _ => unreachable!(),
    }
}

#[test]
fn deal_accepts_participation_observed() {
    // Arrange
    let mut faction = machine_in_phase1();

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 2 }]
    );
}

#[test]
fn deal_accepts_ready_observed() {
    // Arrange
    let mut faction = machine_in_phase1();

    // Act
    let outcomes = match faction.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 2 }]);
}

#[test]
fn deal_accepts_local_participation_completed() {
    // Arrange
    let mut faction = machine_in_phase1();

    // Act
    let outcomes = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0], Outcome::LocalParticipationCompleted);
    assert_eq!(outcomes[1], Outcome::BroadcastLocalReady);
}

#[test]
fn deal_accepts_deadline_expired() {
    // Arrange
    let mut faction = machine_in_phase1();

    // Act
    let outcomes = match faction.process(Command::DeadlineExpired) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ReadinessExited {
            mode: ReadinessExitMode::TimedOut
        }]
    );
}

#[test]
fn participation_observed_non_member() {
    // Arrange
    let mut faction = machine_in_phase1();
    let before = p1(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 99,
        freshness: TIMELY,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(p1(&mut faction), before);
}

#[test]
fn participation_observed_stale() {
    // Arrange
    let mut faction = machine_in_phase1();
    let before = p1(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 2,
        freshness: STALE,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::StaleParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(p1(&mut faction), before);
}

#[test]
fn participation_observed_duplicate() {
    let mut faction = machine_in_phase1();
    let _ = faction.process(Command::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let before = p1(&mut faction);
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateParticipationIgnored { peer_id: 2 }]
    );
    assert_eq!(p1(&mut faction), before);
}

#[test]
fn participation_observed_first_timely() {
    // Arrange
    let mut faction = machine_in_phase1();
    let before = p1(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::ParticipationAccepted { peer_id: 3 }]
    );
    assert_eq!(p1(&mut faction), before + 1);
}

#[test]
fn participation_observed_first_delayed() {
    // Arrange
    let mut faction = machine_in_phase1();
    let before = p1(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ParticipationObserved {
        peer_id: 3,
        freshness: DELAYED,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DelayedParticipationAccepted { peer_id: 3 }]
    );
    assert_eq!(p1(&mut faction), before + 1);
}

#[test]
fn ready_observed_non_member() {
    // Arrange
    let mut faction = machine_in_phase1();
    let before = p2(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved {
        peer_id: 99,
        freshness: TIMELY,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 99 }]);
    assert_eq!(p2(&mut faction), before);
}

#[test]
fn ready_observed_stale() {
    // Arrange
    let mut faction = machine_in_phase1();
    let before = p2(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: STALE,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::StaleReadyIgnored { peer_id: 2 }]);
    assert_eq!(p2(&mut faction), before);
}

#[test]
fn ready_observed_duplicate() {
    // Arrange
    let mut faction = machine_in_phase1();
    let _ = faction.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let before = p2(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateReadyIgnored { peer_id: 2 }]
    );
    assert_eq!(p2(&mut faction), before);
}

#[test]
fn ready_observed_first_timely() {
    // Arrange
    let mut faction = machine_in_phase1();
    let before = p2(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::ReadyAccepted { peer_id: 3 }]);
    assert_eq!(p2(&mut faction), before + 1);
}

#[test]
fn ready_observed_first_delayed() {
    // Arrange
    let mut faction = machine_in_phase1();
    let before = p2(&mut faction);

    // Act
    let outcomes = match faction.process(Command::ReadyObserved {
        peer_id: 3,
        freshness: DELAYED,
        current_marker: MARKER,
    }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(outcomes, vec![Outcome::DelayedReadyAccepted { peer_id: 3 }]);
    assert_eq!(p2(&mut faction), before + 1);
}

#[test]
fn local_completion_no_quorum() {
    // Arrange & Act
    let mut faction = machine_in_phase1();
    let outcomes = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ]
    );
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert_eq!(
        snap.node_state(),
        NodeState::Phase2Active
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
    let _ = faction.process(Command::ParticipationObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = faction.process(Command::ReadyObserved {
        peer_id: 1,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = faction.process(Command::ReadyObserved {
        peer_id: 2,
        freshness: TIMELY,
        current_marker: MARKER,
    });
    let _ = faction.process(Command::ReadyObserved {
        peer_id: 3,
        freshness: TIMELY,
        current_marker: MARKER,
    });

    // Act
    let outcomes = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::ReadyQuorumReached,
            Outcome::ReadinessExited {
                mode: ReadinessExitMode::Bootstrapped,
            },
        ]
    );
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert!(snap.readiness_exited());
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::Bootstrapped));
}

#[test]
fn deadline_expired_in_phase1() {
    // Arrange
    let mut faction = machine_in_phase1();

    // Act & Assert
    let outcomes = match faction.process(Command::DeadlineExpired) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => panic!("expected accepted"),
    };
    assert_eq!(
        outcomes,
        vec![Outcome::ReadinessExited {
            mode: ReadinessExitMode::TimedOut,
        }]
    );
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };
    assert!(snap.readiness_exited());
    assert_eq!(snap.exit_mode(), Some(ReadinessExitMode::TimedOut));
}

#[test]
fn vibe_check_in_phase1() {
    // Arrange & Act
    let mut faction = machine_in_phase1();
    let snap = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(
        snap.node_state(),
        NodeState::Phase1Active
    );
    assert!(!snap.readiness_exited());
    assert!(!snap.local_participation_complete());
    assert_eq!(snap.exit_mode(), None);
    assert_eq!(snap.phase1_confirmed_count(), 1);
    assert_eq!(snap.phase2_confirmed_count(), 0);
    assert_eq!(snap.quorum_threshold(), THRESHOLD);
}

#[test]
fn pinging_cluster_view_inherits_correctly() {
    // Arrange
    let pinging = Pinging::new(5);
    let prev = ClusterView::new(NodeState::Phase2Active, true, 99, 99, 4);

    // Act
    let result = pinging.cluster_view(&prev);

    // Assert
    assert_eq!(
        result.node_state(),
        NodeState::Phase1Active
    );
    assert_eq!(result.phase1_confirmed_count(), 0);
    assert_eq!(result.phase2_confirmed_count(), 0);
    assert_eq!(result.exit_mode(), None);
    assert!(result.local_participation_complete());
    assert!(!result.readiness_exited());
}
