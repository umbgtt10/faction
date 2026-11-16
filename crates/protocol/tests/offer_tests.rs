// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction_protocol::offer::Offer;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::transport_message::TransportMessage;

fn membership(members: Vec<u64>) -> TransportMessage {
    TransportMessage::Membership {
        from: 1,
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
fn command_of_an_offer_is_the_membership_with_the_node() {
    // Arrange
    let offer = Offer::of(&membership(vec![1, 2]), 5).expect("a membership");

    // Act
    let command = offer.command();

    // Assert
    assert_eq!(
        command,
        Command::MembershipReceived {
            members: vec![1, 2, 5],
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
    let config = offer.config(5, 1);

    // Assert
    assert_eq!(config.peers(), &[5, 1]);
}

#[test]
fn config_of_an_offer_is_the_newcomer_s_at_its_epoch() {
    // Arrange
    let offer = Offer::of(&membership(vec![1]), 5).expect("a membership");

    // Act
    let config = offer.config(5, 3);

    // Assert
    assert_eq!(config.peer_id(), 5);
    assert_eq!(config.peers(), &[1, 5]);
    assert_eq!(config.required_count(), 2);
    assert_eq!(config.suspicion_threshold(), 3);
    assert_eq!(config.epoch(), 4);
}

#[test]
fn of_any_other_message_is_none() {
    // Arrange & Act
    let offer = Offer::of(&TransportMessage::Ping { from: 1, epoch: 4 }, 5);

    // Assert
    assert!(offer.is_none());
}
