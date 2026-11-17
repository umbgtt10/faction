// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::answers::Answers;
use alloc::boxed::Box;
use alloc::vec;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::instance::instance_id::InstanceId;
use faction::members::Members;
use faction::no_op_observer::NoOpObserver;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction::state::State;
use faction::states::bootstrapped::Bootstrapped;
use faction::states::reconfiguring::Reconfiguring;

extern crate alloc;

// Node 0 of members 0 to 4, requiring four, concluded, then asked to add
// peer 9.
fn reach_reconfiguring() -> Faction {
    let mut faction = Faction::new(
        Config::new(
            InstanceId {
                node: 0,
                incarnation: 1,
            },
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(4),
            1,
            0,
            false,
        ),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::LocalParticipationCompleted).output;
    let _ = faction
        .process(Command::ReadyObserved {
            peer_id: 1,
            epoch: 0,
        })
        .output;
    let _ = faction
        .process(Command::ReadyObserved {
            peer_id: 2,
            epoch: 0,
        })
        .output;
    let _ = faction
        .process(Command::ReadyObserved {
            peer_id: 3,
            epoch: 0,
        })
        .output;
    let _ = faction
        .process(Command::AdditionRequested { peer_id: 9 })
        .output;
    faction
}

// The same node asked instead to remove member 4, which its count of four can
// spare.
fn reach_removing() -> Faction {
    let mut faction = Faction::new(
        Config::new(
            InstanceId {
                node: 0,
                incarnation: 1,
            },
            vec![0, 1, 2, 3, 4],
            QuorumPolicy::new(4),
            1,
            0,
            false,
        ),
        Box::new(NoOpObserver),
    );
    let _ = faction.process(Command::LocalParticipationCompleted).output;
    let _ = faction
        .process(Command::ReadyObserved {
            peer_id: 1,
            epoch: 0,
        })
        .output;
    let _ = faction
        .process(Command::ReadyObserved {
            peer_id: 2,
            epoch: 0,
        })
        .output;
    let _ = faction
        .process(Command::ReadyObserved {
            peer_id: 3,
            epoch: 0,
        })
        .output;
    let _ = faction
        .process(Command::RemovalRequested { peer_id: 4 })
        .output;
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
        Reconfiguration::AdditionRequested { peer_id: 7 },
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
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 7 }
    );
}

#[test]
fn cluster_view_reports_bootstrapped_with_the_removal_requested() {
    // Arrange
    let mut faction = reach_removing();

    // Act
    let (_, cluster_view, _) = Answers::accepted(faction.process(Command::ProbeDue).output);

    // Assert
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::RemovalRequested { peer_id: 4 }
    );
}

#[test]
fn process_addition_aborted_of_another_peer_is_ignored() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::AdditionAborted { peer_id: 8 })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::AbortIgnored { peer_id: 8 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}

#[test]
fn process_addition_aborted_of_the_newcomer_withdraws_it() {
    // Arrange
    let mut faction = reach_reconfiguring();
    let _ = faction
        .process(Command::MembershipAcknowledged {
            peer_id: 9,
            epoch: 0,
        })
        .output;

    // Act
    let (outcomes, cluster_view, admissible) = Answers::accepted(
        faction
            .process(Command::AdditionAborted { peer_id: 9 })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::AdditionWithdrawn { peer_id: 9 }]);
    assert_eq!(cluster_view.reconfiguration(), Reconfiguration::None);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3, 4]);
    assert!(admissible.contains(&Command::AdditionRequested { peer_id: 0 }));
}

#[test]
fn process_addition_aborted_with_a_removal_in_flight_is_ignored() {
    // Arrange
    let mut faction = reach_removing();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::AdditionAborted { peer_id: 4 })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::AbortIgnored { peer_id: 4 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::RemovalRequested { peer_id: 4 }
    );
}

#[test]
fn process_addition_committed_of_a_member_is_ignored() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::AdditionCommitted {
                peer_id: 1,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::DuplicateMemberIgnored { peer_id: 1 }]
    );
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}

#[test]
fn process_addition_committed_of_another_peer_keeps_the_addition_in_flight() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::AdditionCommitted {
                peer_id: 8,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberAdmitted { peer_id: 8 }]);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3, 4, 8]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}

#[test]
fn process_addition_committed_of_the_newcomer_completes_the_addition() {
    // Arrange
    let mut faction = reach_reconfiguring();
    let _ = faction
        .process(Command::MembershipAcknowledged {
            peer_id: 9,
            epoch: 0,
        })
        .output;

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::AdditionCommitted {
                peer_id: 9,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberAdmitted { peer_id: 9 }]);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3, 4, 9]);
    assert_eq!(cluster_view.reconfiguration(), Reconfiguration::None);
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn process_addition_requested_while_a_removal_is_in_flight_is_rejected() {
    // Arrange
    let mut faction = reach_removing();

    // Act
    let result = faction
        .process(Command::AdditionRequested { peer_id: 9 })
        .output;

    // Assert
    assert!(Answers::is_rejected(&result));
}

#[test]
fn process_addition_requested_while_one_is_in_flight_is_rejected() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let result = faction
        .process(Command::AdditionRequested { peer_id: 8 })
        .output;

    // Assert
    assert!(Answers::is_rejected(&result));
}

#[test]
fn process_join_approved_is_rejected() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let result = faction.process(Command::JoinApproved { peer_id: 8 }).output;

    // Assert
    assert!(Answers::is_rejected(&result));
}

#[test]
fn process_join_requested_and_rejected_are_answered_as_a_concluded_node_answers() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (requested, _, _) = Answers::accepted(
        faction
            .process(Command::JoinRequested { peer_id: 8 })
            .output,
    );
    let (rejected, cluster_view, _) =
        Answers::accepted(faction.process(Command::JoinRejected { peer_id: 8 }).output);

    // Assert
    assert_eq!(requested, vec![Outcome::EmitJoinRequest { peer_id: 8 }]);
    assert_eq!(rejected, vec![Outcome::JoinDenied { peer_id: 8 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}

#[test]
fn process_membership_acknowledged_by_another_peer_is_ignored() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::MembershipAcknowledged {
                peer_id: 8,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::AcknowledgementIgnored { peer_id: 8 }]
    );
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}

#[test]
fn process_membership_acknowledged_by_the_newcomer_proposes_the_addition() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::MembershipAcknowledged {
                peer_id: 9,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::ProposeAddition { peer_id: 9 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionAcknowledged { peer_id: 9 }
    );
}

#[test]
fn process_membership_acknowledged_twice_ignores_the_repeat() {
    // Arrange
    let mut faction = reach_reconfiguring();
    let _ = faction
        .process(Command::MembershipAcknowledged {
            peer_id: 9,
            epoch: 0,
        })
        .output;

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::MembershipAcknowledged {
                peer_id: 9,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::AcknowledgementIgnored { peer_id: 9 }]
    );
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionAcknowledged { peer_id: 9 }
    );
}

#[test]
fn process_membership_acknowledged_with_a_removal_in_flight_is_ignored() {
    // Arrange
    let mut faction = reach_removing();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::MembershipAcknowledged {
                peer_id: 4,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::AcknowledgementIgnored { peer_id: 4 }]
    );
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::RemovalRequested { peer_id: 4 }
    );
}

#[test]
fn process_of_a_membership_abandons_the_change_in_flight() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, view, _) = Answers::accepted(
        faction
            .process(Command::MembershipReceived {
                members: vec![0, 1, 2, 3, 4, 9],
                required_count: 4,
                epoch: 1,
            })
            .output,
    );

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::MembershipAdopted { epoch: 1 },
            Outcome::ChangeAbandoned {
                reconfiguration: Reconfiguration::AdditionRequested { peer_id: 9 }
            },
        ]
    );
    assert_eq!(view.reconfiguration(), Reconfiguration::None);
    assert_eq!(view.members(), &Members::new(vec![0, 1, 2, 3, 4, 9]));
}

#[test]
fn process_of_the_newcomer_s_acknowledgement_from_behind_refuses_it() {
    // Arrange
    let mut faction = reach_reconfiguring();
    let _ = faction
        .process(Command::RequiredCountChanged {
            required_count: 4,
            epoch: 1,
        })
        .output;

    // Act
    let (outcomes, view, _) = Answers::accepted(
        faction
            .process(Command::MembershipAcknowledged {
                peer_id: 9,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(
        outcomes,
        vec![
            Outcome::StaleEpochIgnored {
                peer_id: 9,
                epoch: 0
            },
            Outcome::SendMembership { peer_id: 9 },
        ]
    );
    assert_eq!(
        view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}

#[test]
fn process_participation_of_a_member_acknowledges_its_rejoin() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, _, _) = Answers::accepted(
        faction
            .process(Command::ParticipationObserved {
                peer_id: 1,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::AcknowledgeRejoin { peer_id: 1 }]);
}

#[test]
fn process_participation_of_the_newcomer_is_ignored_as_a_non_member() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, _, _) = Answers::accepted(
        faction
            .process(Command::ParticipationObserved {
                peer_id: 9,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::NonMemberIgnored { peer_id: 9 }]);
}

#[test]
fn process_proposal_declined_of_a_removal_the_quorum_can_no_longer_spare_gives_it_up() {
    // Arrange
    let mut faction = reach_removing();
    let _ = faction
        .process(Command::RemovalCommitted {
            peer_id: 3,
            epoch: 1,
        })
        .output;

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::ProposalDeclined {
                peer_id: 4,
                epoch: 1,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::RemovalRefused { peer_id: 4 }]);
    assert_eq!(cluster_view.reconfiguration(), Reconfiguration::None);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 4]);
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn process_proposal_declined_of_a_removal_the_quorum_can_spare_proposes_it_again() {
    // Arrange
    let mut faction = reach_removing();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::ProposalDeclined {
                peer_id: 4,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::ProposeRemoval { peer_id: 4 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::RemovalRequested { peer_id: 4 }
    );
}

#[test]
fn process_proposal_declined_of_another_peer_is_ignored() {
    // Arrange
    let mut faction = reach_removing();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::ProposalDeclined {
                peer_id: 1,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::DeclineIgnored { peer_id: 1 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::RemovalRequested { peer_id: 4 }
    );
}

#[test]
fn process_proposal_declined_of_the_acknowledged_addition_proposes_it_again() {
    // Arrange
    let mut faction = reach_reconfiguring();
    let _ = faction
        .process(Command::MembershipAcknowledged {
            peer_id: 9,
            epoch: 0,
        })
        .output;

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::ProposalDeclined {
                peer_id: 9,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::ProposeAddition { peer_id: 9 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionAcknowledged { peer_id: 9 }
    );
}

#[test]
fn process_proposal_declined_of_the_addition_not_yet_acknowledged_is_ignored() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::ProposalDeclined {
                peer_id: 9,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::DeclineIgnored { peer_id: 9 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}

#[test]
fn process_removal_aborted_of_another_peer_is_ignored() {
    // Arrange
    let mut faction = reach_removing();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::RemovalAborted { peer_id: 3 })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::AbortIgnored { peer_id: 3 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::RemovalRequested { peer_id: 4 }
    );
}

#[test]
fn process_removal_aborted_of_the_peer_in_flight_withdraws_it() {
    // Arrange
    let mut faction = reach_removing();

    // Act
    let (outcomes, cluster_view, admissible) = Answers::accepted(
        faction
            .process(Command::RemovalAborted { peer_id: 4 })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::RemovalWithdrawn { peer_id: 4 }]);
    assert_eq!(cluster_view.reconfiguration(), Reconfiguration::None);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3, 4]);
    assert!(admissible.contains(&Command::RemovalRequested { peer_id: 0 }));
}

#[test]
fn process_removal_aborted_with_an_addition_in_flight_is_ignored() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::RemovalAborted { peer_id: 9 })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::AbortIgnored { peer_id: 9 }]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}

#[test]
fn process_removal_committed_of_another_peer_keeps_the_addition_in_flight() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::RemovalCommitted {
                peer_id: 4,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberRemoved { peer_id: 4 }]);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3]);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}

#[test]
fn process_removal_committed_of_the_peer_in_flight_completes_the_removal() {
    // Arrange
    let mut faction = reach_removing();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::RemovalCommitted {
                peer_id: 4,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(outcomes, vec![Outcome::MemberRemoved { peer_id: 4 }]);
    assert_eq!(cluster_view.members().as_slice(), &[0, 1, 2, 3]);
    assert_eq!(cluster_view.reconfiguration(), Reconfiguration::None);
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
}

#[test]
fn process_removal_requested_while_a_change_is_in_flight_is_rejected() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let result = faction
        .process(Command::RemovalRequested { peer_id: 1 })
        .output;

    // Assert
    assert!(Answers::is_rejected(&result));
}

#[test]
fn process_required_count_changed_sets_the_count_in_effect() {
    // Arrange
    let mut faction = reach_reconfiguring();

    // Act
    let (outcomes, cluster_view, _) = Answers::accepted(
        faction
            .process(Command::RequiredCountChanged {
                required_count: 3,
                epoch: 0,
            })
            .output,
    );

    // Assert
    assert_eq!(
        outcomes,
        vec![Outcome::RequiredCountSet { required_count: 3 }]
    );
    assert_eq!(cluster_view.required_count(), 3);
    assert_eq!(
        cluster_view.reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 9 }
    );
}
