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
use faction::quorum_policy::QuorumPolicy;
use faction::state::State;
use faction::states::bootstrapped::Bootstrapped;
use faction::states::reconfiguring::Reconfiguring;

extern crate alloc;

// Node 0 of members 0 to 4, requiring four, concluded, then asked to add
// peer 9.
fn reach_reconfiguring() -> Faction {
    let mut faction = Faction::new(
        Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(4)),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::LocalParticipationCompleted);
    let _ = faction.process(Command::ReadyObserved { peer_id: 1 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 2 });
    let _ = faction.process(Command::ReadyObserved { peer_id: 3 });
    let _ = faction.process(Command::AdditionRequested { peer_id: 9 });
    faction
}

#[test]
fn cluster_view_reports_bootstrapped_with_the_addition_requested() {
    // Arrange
    let reconfiguring = Reconfiguring::new(
        Bootstrapped::new(
            Members::new(vec![0, 1, 2]),
            QuorumPolicy::new(2),
            vec![1],
            vec![0, 2],
        ),
        7,
    );
    let previous = ClusterViewBuilder::default()
        .with_peer_state(PeerState::Collecting)
        .build();

    // Act
    let cluster_view = reconfiguring.cluster_view(&previous);

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert!(cluster_view.is_pinging_completed());
    assert_eq!(cluster_view.collecting_peers(), &[0, 2]);
    assert_eq!(cluster_view.required_count(), 2);
    assert_eq!(cluster_view.addition(), Addition::Requested { peer_id: 7 });
}

#[test]
fn process_addition_aborted_of_another_peer_is_ignored() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::AdditionAborted { peer_id: 8 }));

    // Assert
    assert_eq!(outcomes, vec![Outcome::AbortIgnored { peer_id: 8 }]);
    assert_eq!(cluster_view.addition(), Addition::Requested { peer_id: 9 });
}

#[test]
fn process_addition_aborted_of_the_newcomer_withdraws_it() {
    // Arrange
    let mut faction = reach_reconfiguring();
    let _ = faction.process(Command::MembershipAcknowledged { peer_id: 9 });

    // Act
    let (outcomes, cluster_view, admissible) =
        Answers::accepted(faction.process(Command::AdditionAborted { peer_id: 9 }));

    // Assert
    assert_eq!(outcomes, vec![Outcome::AdditionWithdrawn { peer_id: 9 }]);
    assert_eq!(cluster_view.addition(), Addition::None);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3, 4]);
    assert!(admissible.contains(&Command::AdditionRequested { peer_id: 0 }));
}

#[test]
fn process_addition_committed_of_a_member_is_ignored() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::AdditionCommitted { peer_id: 1 }));

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateMemberIgnored { peer_id: 1 }]
    );
    assert_eq!(cluster_view.addition(), Addition::Requested { peer_id: 9 });
}

#[test]
fn process_addition_committed_of_another_peer_keeps_the_addition_in_flight() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::AdditionCommitted { peer_id: 8 }));

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberAdmitted { peer_id: 8 }]);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3, 4, 8]);
    assert_eq!(cluster_view.addition(), Addition::Requested { peer_id: 9 });
}

#[test]
fn process_addition_committed_of_the_newcomer_completes_the_addition() {
    // Arrange
    let mut faction = reach_reconfiguring();
    let _ = faction.process(Command::MembershipAcknowledged { peer_id: 9 });

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::AdditionCommitted { peer_id: 9 }));

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberAdmitted { peer_id: 9 }]);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3, 4, 9]);
    assert_eq!(cluster_view.addition(), Addition::None);
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn process_addition_requested_while_one_is_in_flight_is_rejected() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let result = faction.process(Command::AdditionRequested { peer_id: 8 });

    // Assert
    assert!(Answers::is_rejected(&result));
}

#[test]
fn process_join_approved_is_rejected() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let result = faction.process(Command::JoinApproved { peer_id: 8 });

    // Assert
    assert!(Answers::is_rejected(&result));
}

#[test]
fn process_join_requested_and_rejected_are_answered_as_a_concluded_node_answers() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (requested, _, _) =
        Answers::accepted(faction.process(Command::JoinRequested { peer_id: 8 }));
    let (rejected, cluster_view, _) =
        Answers::accepted(faction.process(Command::JoinRejected { peer_id: 8 }));

    // Assert
    assert_eq!(requested, vec![Outcome::EmitJoinRequest { peer_id: 8 }]);
    assert_eq!(rejected, vec![Outcome::JoinDenied { peer_id: 8 }]);
    assert_eq!(cluster_view.addition(), Addition::Requested { peer_id: 9 });
}

#[test]
fn process_membership_acknowledged_by_another_peer_is_ignored() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::MembershipAcknowledged { peer_id: 8 }));

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::AcknowledgementIgnored { peer_id: 8 }]
    );
    assert_eq!(cluster_view.addition(), Addition::Requested { peer_id: 9 });
}

#[test]
fn process_membership_acknowledged_by_the_newcomer_proposes_the_addition() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::MembershipAcknowledged { peer_id: 9 }));

    // Assert
    assert_eq!(outcomes, vec![Outcome::ProposeAddition { peer_id: 9 }]);
    assert_eq!(
        cluster_view.addition(),
        Addition::Acknowledged { peer_id: 9 }
    );
}

#[test]
fn process_membership_acknowledged_twice_ignores_the_repeat() {
    // Arrange
    let mut faction = reach_reconfiguring();
    let _ = faction.process(Command::MembershipAcknowledged { peer_id: 9 });

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::MembershipAcknowledged { peer_id: 9 }));

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::AcknowledgementIgnored { peer_id: 9 }]
    );
    assert_eq!(
        cluster_view.addition(),
        Addition::Acknowledged { peer_id: 9 }
    );
}

#[test]
fn process_participation_of_a_member_acknowledges_its_rejoin() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, _, _) =
        Answers::accepted(faction.process(Command::ParticipationObserved { peer_id: 1 }));

    // Assert
    assert_eq!(outcomes, vec![Outcome::AcknowledgeRejoin { peer_id: 1 }]);
}

#[test]
fn process_participation_of_the_newcomer_is_ignored_as_a_non_member() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, _, _) =
        Answers::accepted(faction.process(Command::ParticipationObserved { peer_id: 9 }));

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 9 }]);
}

#[test]
fn process_required_count_changed_sets_the_count_in_effect() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) =
        Answers::accepted(faction.process(Command::RequiredCountChanged { required_count: 3 }));

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::RequiredCountSet { required_count: 3 }]
    );
    assert_eq!(cluster_view.required_count(), 3);
    assert_eq!(cluster_view.addition(), Addition::Requested { peer_id: 9 });
}
