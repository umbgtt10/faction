// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::answers::Answers;
use alloc::boxed::Box;
use alloc::vec;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::members::Members;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum::Quorum;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction::stage::Stage;
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

fn config() -> Config {
    Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4), 1)
}

extern crate alloc;

// The same node, requiring all three.
fn needing_all_three() -> Bootstrapped {
    Bootstrapped::new(
        Members::new(vec![0, 1, 2]),
        QuorumPolicy::new(3),
        vec![1],
        vec![0, 1],
    )
}

fn reach_bootstrapped() -> Faction {
    let mut faction = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4), 1),
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
        .answer(Command::AdditionCommitted { peer_id: 9 }, &config())
        .expect("a concluded node applies a commit");

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberAdmitted { peer_id: 9 }]);
    let cluster_view = answered.cluster_view(&ClusterViewBuilder::default().build());
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 9]);
}

#[test]
fn answer_of_a_committed_removal_takes_the_member_out_of_the_members_and_the_peers_gathered() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let (outcomes, answered) = bootstrapped
        .answer(Command::RemovalCommitted { peer_id: 1 }, &config())
        .expect("a concluded node applies a commit");

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberRemoved { peer_id: 1 }]);
    let cluster_view = answered.cluster_view(&ClusterViewBuilder::default().build());
    assert_eq!(cluster_view.members().as_slice(), &[0, 2]);
    assert!(cluster_view.pinging_peers().is_empty());
    assert_eq!(cluster_view.collecting_peers(), &[0]);
    assert_eq!(cluster_view.quorum(), Quorum::Held);
}

#[test]
fn answer_of_a_committed_removal_the_count_needs_loses_the_quorum() {
    // Arrange
    let bootstrapped = needing_all_three();

    // Act
    let (outcomes, answered) = bootstrapped
        .answer(Command::RemovalCommitted { peer_id: 1 }, &config())
        .expect("a concluded node applies a commit");

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::MemberRemoved { peer_id: 1 }, Outcome::QuorumLost]
    );
    let cluster_view = answered.cluster_view(&ClusterViewBuilder::default().build());
    assert_eq!(cluster_view.quorum(), Quorum::Lost);
}

#[test]
fn answer_of_a_count_past_the_members_loses_the_quorum() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let (outcomes, answered) = bootstrapped
        .answer(
            Command::RequiredCountChanged { required_count: 4 },
            &config(),
        )
        .expect("a concluded node takes a new count");

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::RequiredCountSet { required_count: 4 },
            Outcome::QuorumLost
        ]
    );
    let cluster_view = answered.cluster_view(&ClusterViewBuilder::default().build());
    assert_eq!(cluster_view.quorum(), Quorum::Lost);
}

#[test]
fn answer_of_a_member_s_participation_acknowledges_its_rejoin() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let (outcomes, answered) = bootstrapped
        .answer(Command::ParticipationObserved { peer_id: 2 }, &config())
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
fn answer_of_a_probe_period_names_the_member_to_probe() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let (outcomes, answered) = bootstrapped
        .answer(Command::ProbeDue, &config())
        .expect("a concluded node tracks its members' liveness");

    // Assert
    assert_eq!(outcomes, vec![Outcome::SendProbe { peer_id: 1 }]);
    let cluster_view = answered.cluster_view(&ClusterViewBuilder::default().build());
    assert_eq!(cluster_view.last_probed(), Some(1));
    assert_eq!(cluster_view.quorum(), Quorum::Held);
}

#[test]
fn answer_of_a_removal_request_is_none() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let answered = bootstrapped.answer(Command::RemovalRequested { peer_id: 1 }, &config());

    // Assert
    assert!(answered.is_none());
}

#[test]
fn answer_of_an_addition_request_is_none() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let answered = bootstrapped.answer(Command::AdditionRequested { peer_id: 9 }, &config());

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
    assert_eq!(cluster_view.reconfiguration(), Reconfiguration::None);
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
    assert_eq!(cluster_view.reconfiguration(), Reconfiguration::None);
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
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
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

// Only the node's own removal removes it: another member's, committed, is a
// membership decision it applies and stays concluded.
#[test]
fn step_removal_committed_of_another_member_takes_it_out_still_bootstrapped() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let (outcomes, stage) = bootstrapped
        .step(Command::RemovalCommitted { peer_id: 1 }, &config())
        .expect("a concluded node applies a commit");

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberRemoved { peer_id: 1 }]);
    let cluster_view = stage.cluster_view(&ClusterViewBuilder::default().build());
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(cluster_view.members().as_slice(), &[0, 2]);
}

#[test]
fn step_removal_requested_of_a_member_the_quorum_can_spare_proposes_it_still_bootstrapped() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let (outcomes, stage) = bootstrapped
        .step(Command::RemovalRequested { peer_id: 1 }, &config())
        .expect("a concluded node is asked for a removal");

    // Assert
    assert_eq!(outcomes, vec![Outcome::ProposeRemoval { peer_id: 1 }]);
    let cluster_view = stage.cluster_view(&ClusterViewBuilder::default().build());
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::RemovalRequested { peer_id: 1 }
    );
}

#[test]
fn step_removal_requested_of_a_non_member_is_ignored() {
    // Arrange
    let bootstrapped = concluded();

    // Act
    let (outcomes, stage) = bootstrapped
        .step(Command::RemovalRequested { peer_id: 9 }, &config())
        .expect("a concluded node is asked for a removal");

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 9 }]);
    assert!(matches!(stage, Stage::Bootstrapped(_)));
}

#[test]
fn step_removal_requested_of_a_suspect_is_proposed_with_the_quorum_lost() {
    // Arrange
    let (_, suspecting) = needing_all_three()
        .answer(Command::IndirectProbesMissed { peer_id: 2 }, &config())
        .expect("a concluded node tracks its members' liveness");

    // Act
    let (outcomes, stage) = suspecting
        .step(Command::RemovalRequested { peer_id: 2 }, &config())
        .expect("a concluded node is asked for a removal");

    // Assert
    assert_eq!(outcomes, vec![Outcome::ProposeRemoval { peer_id: 2 }]);
    assert!(matches!(stage, Stage::Reconfiguring(_)));
}

#[test]
fn step_removal_requested_the_quorum_cannot_spare_is_refused() {
    // Arrange
    let bootstrapped = needing_all_three();

    // Act
    let refusals: Vec<Vec<Outcome>> = [1, 0]
        .map(|peer_id| {
            bootstrapped
                .step(Command::RemovalRequested { peer_id }, &config())
                .expect("a concluded node is asked for a removal")
                .0
        })
        .to_vec();

    // Assert
    assert_eq!(
        refusals,
        vec![
            vec![Outcome::RemovalRefused { peer_id: 1 }],
            vec![Outcome::RemovalRefused { peer_id: 0 }],
        ]
    );
}
