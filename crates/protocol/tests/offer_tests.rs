// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::cluster_view::ClusterView;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::instance::instance_id::InstanceId;
use faction::peer_state::PeerState;
use faction_protocol::offer::Offer;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::transport_message::TransportMessage;

// A running node's view at `epoch`.
fn at_epoch(epoch: u64) -> ClusterView {
    ClusterViewBuilder::from_view_at(&ClusterViewBuilder::default().build(), epoch, false).build()
}

fn membership(members: Vec<u64>) -> TransportMessage {
    TransportMessage::Membership {
        from: 1,
        to: 5,
        members,
        required_count: 2,
        epoch: 4,
    }
}

#[test]
fn acknowledged_of_what_taking_asked_acknowledges_first_and_drops_the_noops() {
    // Arrange
    let offer = Offer::of(&membership(vec![1]), 5).expect("a membership");

    // Act
    let outputs = offer.acknowledged(
        vec![
            OutputMessage::Noop,
            OutputMessage::BroadcastPing { epoch: 4 },
        ],
        5,
        4,
    );

    // Assert
    assert_eq!(
        outputs,
        vec![
            OutputMessage::Send {
                to: 1,
                message: TransportMessage::MembershipAcknowledged { from: 5, epoch: 4 },
            },
            OutputMessage::BroadcastPing { epoch: 4 },
        ]
    );
}

#[test]
fn answered_by_a_node_it_removed_acknowledges_nothing() {
    // Arrange
    let offer = Offer::of(&membership(vec![1, 2]), 5).expect("a membership");
    let removed = ClusterViewBuilder::from_view_at(&at_epoch(4), 4, false)
        .with_peer_state(PeerState::Removed)
        .build();

    // Act
    let outputs = offer.answered(vec![OutputMessage::Noop], 5, &removed);

    // Assert
    assert_eq!(outputs, vec![OutputMessage::Noop]);
}

#[test]
fn answered_by_a_node_still_running_acknowledges_at_its_epoch() {
    // Arrange
    let offer = Offer::of(&membership(vec![1, 5]), 5).expect("a membership");

    // Act
    let outputs = offer.answered(
        vec![OutputMessage::BroadcastPing { epoch: 4 }],
        5,
        &at_epoch(4),
    );

    // Assert
    assert_eq!(
        outputs,
        vec![
            OutputMessage::Send {
                to: 1,
                message: TransportMessage::MembershipAcknowledged { from: 5, epoch: 4 },
            },
            OutputMessage::BroadcastPing { epoch: 4 },
        ]
    );
}

#[test]
fn command_of_an_offer_is_the_membership_as_sent() {
    // Arrange
    let offer = Offer::of(&membership(vec![1, 2]), 5).expect("a membership");

    // Act
    let command = offer.command();

    // Assert
    assert_eq!(
        command,
        Command::MembershipReceived {
            members: vec![1, 2],
            required_count: 2,
            epoch: 4,
        }
    );
}

#[test]
fn config_of_an_offer_already_naming_the_node_lists_it_once() {
    // Arrange
    let offer = Offer::of(&membership(vec![5, 1]), 5).expect("a membership");

    // Act
    let config = offer.config(
        InstanceId {
            node: 5,
            incarnation: 1,
        },
        1,
        1,
    );

    // Assert
    assert_eq!(config.peers(), &[5, 1]);
}

#[test]
fn config_of_an_offer_is_the_newcomer_s_at_its_epoch() {
    // Arrange
    let offer = Offer::of(&membership(vec![1]), 5).expect("a membership");

    // Act
    let config = offer.config(
        InstanceId {
            node: 5,
            incarnation: 2,
        },
        3,
        2,
    );

    // Assert
    assert_eq!(
        config.instance(),
        InstanceId {
            node: 5,
            incarnation: 2
        }
    );
    assert_eq!(config.peer_id(), 5);
    assert_eq!(config.peers(), &[1, 5]);
    assert_eq!(config.required_count(), 2);
    assert_eq!(config.suspicion_threshold(), 3);
    assert_eq!(config.changes_in_flight(), 2);
    assert_eq!(config.epoch(), 4);
}

#[test]
fn of_a_membership_addressed_to_another_peer_is_no_offer() {
    // Arrange
    let addressed_to_six = TransportMessage::Membership {
        from: 1,
        to: 6,
        members: vec![1],
        required_count: 1,
        epoch: 4,
    };

    // Act
    let offer = Offer::of(&addressed_to_six, 5);

    // Assert
    assert!(offer.is_none());
}

#[test]
fn of_any_other_message_is_none() {
    // Arrange & Act
    let offer = Offer::of(&TransportMessage::Ping { from: 1, epoch: 4 }, 5);

    // Assert
    assert!(offer.is_none());
}
