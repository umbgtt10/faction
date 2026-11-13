// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::answers::Answers;
use alloc::boxed::Box;
use alloc::vec;
use faction::addition::Addition;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::members::Members;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;
use faction::state::State;
use faction::states::bootstrapped::Bootstrapped;

// A concluded node of members 0, 1 and 2, requiring two.
fn concluded() -> Bootstrapped {
    Bootstrapped::new(
        Members::new(vec![0, 1, 2]),
        QuorumPolicy::new(2),
        vec![1],
        vec![0, 1],
    )
}

extern crate alloc;

fn reach_bootstrapped() -> Faction {
    let mut faction = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::ParticipationObserved { peer_id: 1 });
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 3 });
    faction
}

#[test]
fn answer_of_a_committed_addition_admits_the_peer() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let (outcomes, answered) = bootstrapped
        .answer(Command::AdditionCommitted { peer_id: 9 })
        .expect("a concluded node applies a commit");

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberAdmitted { peer_id: 9 }]);
    let cluster_view = answered.cluster_view(&ClusterViewBuilder::default().build());
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 9]);
}

#[test]
fn answer_of_a_member_s_participation_acknowledges_its_rejoin() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let (outcomes, answered) = bootstrapped
        .answer(Command::ParticipationObserved { peer_id: 2 })
        .expect("a concluded node answers a member still bootstrapping");

    // Assert
    assert_eq!(outcomes, vec![Outcome::AcknowledgeRejoin { peer_id: 2 }]);
    let previous = ClusterViewBuilder::default().build();
    assert_eq!(
        answered.cluster_view(&previous),
        bootstrapped.cluster_view(&previous)
    );
}

#[test]
fn answer_of_an_addition_request_is_none() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let answered = bootstrapped.answer(Command::AdditionRequested { peer_id: 9 });

    // Assert
    assert!(answered.is_none());
}

#[test]
fn cluster_view_overrides_all_fields() {
    // Arrange
    let bootstrapped = Bootstrapped::new(
        Members::new(vec![1, 2, 3, 4, 5]),
        QuorumPolicy::new(4),
        vec![1, 2],
        vec![1, 2, 3, 4, 5],
    );
    let prev = ClusterViewBuilder::default()
        .with_peer_state(PeerState::Pinging)
        .with_pinging_peers(vec![1, 2])
        .with_collecting_peers(vec![1, 2, 3, 4, 5])
        .with_required_count(4)
        .build();

    // Act
    let result = bootstrapped.cluster_view(&prev);

    // Assert
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert!(result.is_pinging_completed());
    assert_eq!(result.peer_state(), PeerState::Bootstrapped);
    assert_eq!(result.pinging_peers(), &[1, 2]);
    assert_eq!(result.collecting_peers(), &[1, 2, 3, 4, 5]);
    assert_eq!(result.required_count(), 4);
}

#[test]
fn process_addition_committed_admits_the_peer() {
    // Arrange
    let mut faction = reach_bootstrapped();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::AdditionCommitted { peer_id: 9 }));

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberAdmitted { peer_id: 9 }]);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3, 4, 9]);
    assert_eq!(cluster_view.addition(), Addition::None);
}

#[test]
fn process_addition_requested_of_a_member_is_ignored() {
    // Arrange
    let mut faction = reach_bootstrapped();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::AdditionRequested { peer_id: 2 }));

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateMemberIgnored { peer_id: 2 }]
    );
    assert_eq!(cluster_view.addition(), Addition::None);
}

#[test]
fn process_addition_requested_of_a_newcomer_reconfigures_still_bootstrapped() {
    // Arrange
    let mut faction = reach_bootstrapped();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::AdditionRequested { peer_id: 9 }));

    // Assert
    assert_eq!(outcomes, vec![Outcome::SendMembership { peer_id: 9 }]);
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.addition(), Addition::Requested { peer_id: 9 });
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3, 4]);
}

#[test]
fn process_all_inputs_leave_state_unchanged() {
    // Arrange
    let mut faction = reach_bootstrapped();
    let snapshot_before = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Act
    let r1 = match faction.process(Command::ParticipationObserved { peer_id: 0 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r2 = match faction.process(Command::ReadyObserved { peer_id: 4 }) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r3 = match faction.process(Command::LocalParticipationCompleted) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let r4 = match faction.process(Command::DeadlineExpired) {
        ProcessResult::Accepted { outcomes, .. } => outcomes,
        ProcessResult::Probed { .. } => unreachable!(),
        ProcessResult::Rejected { .. } => vec![],
    };
    let snapshot_after = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(r1, vec![Outcome::AcknowledgeRejoin { peer_id: 0 }]);
    assert!(r2.is_empty());
    assert!(r3.is_empty());
    assert!(r4.is_empty());
    assert_eq!(snapshot_before, snapshot_after);
}

#[test]
fn process_probe_returns_correct_snapshot() {
    // Arrange & Act
    let mut faction = reach_bootstrapped();
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert!(cluster_view.is_pinging_completed());
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.pinging_peers().len(), 1);
    assert_eq!(cluster_view.collecting_peers().len(), 4);
    assert_eq!(cluster_view.required_count(), 4);
}

#[test]
fn process_rejects_participation_observed() {
    // Arrange & Act
    let mut faction = reach_bootstrapped();
    let cluster_view = match faction.process(Command::Probe) {
        ProcessResult::Probed { cluster_view, .. } => cluster_view,
        _ => unreachable!(),
    };

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn process_required_count_changed_sets_the_count_in_effect() {
    // Arrange
    let mut faction = reach_bootstrapped();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::RequiredCountChanged { required_count: 2 }));

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::RequiredCountSet { required_count: 2 }]
    );
    assert_eq!(cluster_view.required_count(), 2);
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
}
