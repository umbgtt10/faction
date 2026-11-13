// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::no_op_observer::NoOpObserver;
use faction_protocol::newcomer::Newcomer;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

fn newcomer() -> Newcomer {
    Newcomer::new(5, Box::new(NoOpObserver))
}

#[test]
fn join_of_a_membership_already_naming_the_newcomer_lists_it_once() {
    // Arrange
    let membership = TransportMessage::Membership {
        from: 0,
        members: vec![0, 1, 5],
        required_count: 2,
    };

    // Act
    let Ok((mut protocol, _)) = newcomer().join(membership) else {
        panic!("a newcomer joins from its membership");
    };

    // Assert
    assert_eq!(protocol.cluster_view().members().as_slice(), &[0, 1, 5]);
}

#[test]
fn join_of_a_membership_becomes_a_node_of_it_acknowledges_and_starts() {
    // Arrange
    let membership = TransportMessage::Membership {
        from: 0,
        members: vec![0, 1],
        required_count: 2,
    };

    // Act
    let Ok((mut protocol, outputs)) = newcomer().join(membership) else {
        panic!("a newcomer joins from its membership");
    };

    // Assert
    assert_eq!(
        outputs,
        vec![
            OutputMessage::Send {
                to: 0,
                message: TransportMessage::MembershipAcknowledged { from: 5 },
            },
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ParticipationObserved {
                peer_id: 0
            })),
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ParticipationObserved {
                peer_id: 1
            })),
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::LocalParticipationCompleted)),
            OutputMessage::BroadcastPing,
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryPing)),
        ]
    );
    let view = protocol.cluster_view();
    assert_eq!(view.members().as_slice(), &[0, 1, 5]);
    assert_eq!(view.required_count(), 2);
}

#[test]
fn join_of_any_other_message_keeps_waiting() {
    // Arrange & Act
    let joined = newcomer().join(TransportMessage::Ping { from: 0 });

    // Assert
    assert!(joined.is_err());
}
