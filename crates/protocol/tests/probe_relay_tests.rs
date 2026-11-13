// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::output_message::OutputMessage;
use faction_protocol::probe_relay::ProbeRelay;
use faction_protocol::transport_message::TransportMessage;

#[test]
fn reply_of_a_probe_answers_its_sender_for_whom_it_was_sent() {
    // Arrange
    let probe = TransportMessage::Probe {
        from: 2,
        relay_for: Some(3),
    };

    // Act
    let outputs = ProbeRelay::reply(1, &probe);

    // Assert
    assert_eq!(
        outputs,
        vec![OutputMessage::Send {
            to: 2,
            message: TransportMessage::ProbeAnswer {
                from: 1,
                relay_for: Some(3),
            },
        }]
    );
}

#[test]
fn reply_of_a_relayed_answer_forwards_it_to_the_node_that_asked() {
    // Arrange
    let answer = TransportMessage::ProbeAnswer {
        from: 4,
        relay_for: Some(3),
    };

    // Act
    let outputs = ProbeRelay::reply(1, &answer);

    // Assert
    assert_eq!(
        outputs,
        vec![OutputMessage::Send {
            to: 3,
            message: TransportMessage::IndirectProbeAnswer { from: 1, target: 4 },
        }]
    );
}

#[test]
fn reply_of_an_indirect_probe_probes_the_target_for_the_node_that_asked() {
    // Arrange
    let request = TransportMessage::IndirectProbe { from: 3, target: 4 };

    // Act
    let outputs = ProbeRelay::reply(1, &request);

    // Assert
    assert_eq!(
        outputs,
        vec![OutputMessage::Send {
            to: 4,
            message: TransportMessage::Probe {
                from: 1,
                relay_for: Some(3),
            },
        }]
    );
}

#[test]
fn reply_of_anything_else_is_noop() {
    // Arrange
    let ping = TransportMessage::Ping { from: 3 };

    // Act
    let outputs = ProbeRelay::reply(1, &ping);

    // Assert
    assert_eq!(outputs, vec![OutputMessage::Noop]);
}
