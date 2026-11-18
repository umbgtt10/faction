// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use faction::cluster_view::ClusterView;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::members::Members;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::stage::Stage;
use faction::state::State;
use faction::states::adoption::Adoption;

fn adoption(members: &[u64]) -> Adoption {
    Adoption::of(&Command::MembershipReceived {
        members: members.to_vec(),
        required_count: 2,
        epoch: 3,
    })
    .expect("a membership")
}

extern crate alloc;

fn view_of(stage: &Stage) -> ClusterView {
    stage.cluster_view(&ClusterViewBuilder::default().build())
}

#[test]
fn of_anything_but_a_membership_is_none() {
    // Arrange & Act
    let adoption = Adoption::<u64>::of(&Command::Probe);

    // Assert
    assert!(adoption.is_none());
}

#[test]
fn removal_of_a_membership_naming_the_node_is_none() {
    // Arrange
    let adoption = adoption(&[2, 0, 1]);

    // Act
    let removal = adoption.removal(0);

    // Assert
    assert!(removal.is_none());
}

#[test]
fn removal_of_a_membership_not_naming_the_node_removes_it_holding_the_membership() {
    // Arrange
    let adoption = adoption(&[1, 2]);

    // Act
    let (outcomes, stage) = adoption.removal(0).expect("the node is not named");

    // Assert
    let view = view_of(&stage);
    assert_eq!(
        outcomes,
        vec![
            Outcome::MembershipAdopted { epoch: 3 },
            Outcome::MemberRemoved { peer_id: 0 },
        ]
    );
    assert_eq!(view.peer_state(), PeerState::Removed);
    assert_eq!(view.members(), &Members::new(vec![1, 2]));
    assert_eq!(view.required_count(), 2);
}

#[test]
fn taken_back_of_a_membership_naming_the_node_is_initial_holding_it() {
    // Arrange
    let adoption = adoption(&[1, 0]);

    // Act
    let (outcomes, stage) = adoption.taken_back(0);

    // Assert
    let view = view_of(&stage);
    assert_eq!(outcomes, vec![Outcome::MembershipAdopted { epoch: 3 }]);
    assert_eq!(view.peer_state(), PeerState::Fresh);
    assert_eq!(view.members(), &Members::new(vec![1, 0]));
}

#[test]
fn taken_back_of_a_membership_not_naming_the_node_leaves_it_removed_holding_it() {
    // Arrange
    let adoption = adoption(&[1, 2]);

    // Act
    let (outcomes, stage) = adoption.taken_back(0);

    // Assert
    let view = view_of(&stage);
    assert_eq!(outcomes, vec![Outcome::MembershipAdopted { epoch: 3 }]);
    assert_eq!(view.peer_state(), PeerState::Removed);
    assert_eq!(view.members(), &Members::new(vec![1, 2]));
}
