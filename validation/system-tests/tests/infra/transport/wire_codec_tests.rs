// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::transport::wire_codec::WireCodec;

fn every_message() -> Vec<TransportMessage> {
    vec![
        TransportMessage::Ping { from: 1 },
        TransportMessage::Ready { from: u64::MAX },
        TransportMessage::Bootstrapped { from: 7 },
        TransportMessage::Membership {
            from: 0,
            members: vec![0, 1, u64::MAX],
            required_count: 2,
        },
        TransportMessage::Membership {
            from: 4,
            members: vec![],
            required_count: 0,
        },
        TransportMessage::MembershipAcknowledged { from: 5 },
        TransportMessage::Probe {
            from: 1,
            relay_for: None,
        },
        TransportMessage::Probe {
            from: 2,
            relay_for: Some(u64::MAX),
        },
        TransportMessage::ProbeAnswer {
            from: 3,
            relay_for: None,
        },
        TransportMessage::ProbeAnswer {
            from: 4,
            relay_for: Some(0),
        },
        TransportMessage::IndirectProbe { from: 5, target: 6 },
        TransportMessage::IndirectProbeAnswer {
            from: 7,
            target: u64::MAX,
        },
    ]
}

#[test]
fn decode_of_a_frame_cut_short_is_none() {
    // Arrange
    let frame = WireCodec::encode(&TransportMessage::Ready { from: 3 });

    // Act
    let decoded = WireCodec::decode(&frame[..frame.len() - 1]);

    // Assert
    assert_eq!(decoded, None);
}

#[test]
fn decode_of_a_frame_with_an_unknown_tag_is_none() {
    // Arrange
    let mut frame = WireCodec::encode(&TransportMessage::Ping { from: 3 });
    frame[12] = 0xff;

    // Act
    let decoded = WireCodec::decode(&frame);

    // Assert
    assert_eq!(decoded, None);
}

#[test]
fn decode_of_a_membership_whose_members_overrun_the_frame_is_none() {
    // Arrange
    let mut frame = WireCodec::encode(&TransportMessage::Membership {
        from: 0,
        members: vec![1],
        required_count: 1,
    });
    frame[21] = 2;

    // Act
    let decoded = WireCodec::decode(&frame);

    // Assert
    assert_eq!(decoded, None);
}

#[test]
fn decode_of_a_probe_whose_relay_flag_is_neither_set_nor_clear_is_none() {
    // Arrange
    let mut frame = WireCodec::encode(&TransportMessage::Probe {
        from: 3,
        relay_for: None,
    });
    frame[13] = 2;

    // Act
    let decoded = WireCodec::decode(&frame);

    // Assert
    assert_eq!(decoded, None);
}

#[test]
fn decode_of_an_indirect_probe_whose_target_is_cut_short_is_none() {
    // Arrange
    let mut frame = WireCodec::encode(&TransportMessage::IndirectProbe { from: 3, target: 4 });
    frame.pop();
    frame[0] -= 1;

    // Act
    let decoded = WireCodec::decode(&frame);

    // Assert
    assert_eq!(decoded, None);
}

#[test]
fn decode_of_each_encoded_message_is_that_message() {
    // Arrange
    let messages = every_message();

    // Act
    let decoded: Vec<Option<TransportMessage>> = messages
        .iter()
        .map(|message| WireCodec::decode(&WireCodec::encode(message)))
        .collect();

    // Assert
    assert_eq!(decoded, messages.into_iter().map(Some).collect::<Vec<_>>());
}

#[test]
fn encode_of_a_message_opens_with_the_length_of_the_rest() {
    // Arrange & Act
    let frame = WireCodec::encode(&TransportMessage::Bootstrapped { from: 9 });

    // Assert
    let length = u32::from_le_bytes(frame[..4].try_into().unwrap());
    assert_eq!(length as usize, frame.len() - 4);
    assert_eq!(&frame[4..12], &9u64.to_le_bytes());
}
