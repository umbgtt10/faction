// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;
use faction_system_tests::infra::transport::frame_buffer::FrameBuffer;
use faction_system_tests::infra::transport::wire_codec::WireCodec;

#[test]
fn next_frame_of_a_frame_pushed_in_two_parts_waits_for_the_whole() {
    // Arrange
    let frame = WireCodec::encode(&TransportMessage::Ping { from: 4 });
    let mut buffer = FrameBuffer::new();
    buffer.push(&frame[..6]);
    let before = buffer.next_frame();

    // Act
    buffer.push(&frame[6..]);
    let after = buffer.next_frame();

    // Assert
    assert_eq!(before, None);
    assert_eq!(after, Some(frame));
}

#[test]
fn next_frame_of_an_empty_buffer_is_none() {
    // Arrange
    let mut buffer = FrameBuffer::new();

    // Act
    let frame = buffer.next_frame();

    // Assert
    assert_eq!(frame, None);
}

#[test]
fn next_frame_of_two_frames_pushed_at_once_returns_each_in_turn() {
    // Arrange
    let first = WireCodec::encode(&TransportMessage::Ping { from: 1 });
    let second = WireCodec::encode(&TransportMessage::Ready { from: 2 });
    let mut buffer = FrameBuffer::new();
    buffer.push(&[first.clone(), second.clone()].concat());

    // Act
    let frames = [
        buffer.next_frame(),
        buffer.next_frame(),
        buffer.next_frame(),
    ];

    // Assert
    assert_eq!(frames, [Some(first), Some(second), None]);
}
