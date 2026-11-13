// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::support::SharedOutput;
use faction_system_tests::process_node::process_channel::ProcessChannel;
use std::sync::mpsc::channel;

#[test]
fn send_writes_the_line_and_waits_for_its_ack() {
    // Arrange
    let output = SharedOutput::default();
    let (ack, acks) = channel();
    ack.send(()).unwrap();
    let process = ProcessChannel::new(Box::new(output.clone()), acks);

    // Act
    process.send("admit 3");

    // Assert
    assert_eq!(output.text(), "admit 3\n");
}
