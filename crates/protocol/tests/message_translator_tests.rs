// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::cluster_view::ClusterView;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::conclusion::Conclusion;
use faction::members::Members;
use faction::outcome::Outcome;
use faction_protocol::input_message::InputMessage;
use faction_protocol::message_translator::MessageTranslator;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

fn defaulted<T: Default>() -> T {
    T::default()
}

fn translator() -> MessageTranslator {
    MessageTranslator::new()
}

// The view a result carries: members 0, 1 and 2, requiring two.
fn view() -> ClusterView {
    ClusterViewBuilder::default()
        .with_members(Members::new(vec![0, 1, 2]))
        .with_required_count(2)
        .build()
}

#[test]
fn default_translates_as_new_does() {
    // Arrange
    let translator: MessageTranslator = defaulted();

    // Act
    let command = translator.to_command(InputMessage::Timer(TimerMessage::DeadlineExpired));

    // Assert
    assert_eq!(command, Command::DeadlineExpired);
}

#[test]
fn to_command_timer_deadline_expired_maps_to_deadline_expired() {
    // Arrange
    let translator = translator();

    // Act
    let command = translator.to_command(InputMessage::Timer(TimerMessage::DeadlineExpired));

    // Assert
    assert_eq!(command, Command::DeadlineExpired);
}

#[test]
fn to_command_timer_local_participation_completed_maps_to_local_participation_completed() {
    // Arrange
    let translator = translator();

    // Act
    let command = translator.to_command(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));

    // Assert
    assert_eq!(command, Command::LocalParticipationCompleted);
}

#[test]
fn to_command_timer_participation_observed_maps_to_participation_observed() {
    // Arrange
    let translator = translator();

    // Act
    let command = translator.to_command(InputMessage::Timer(TimerMessage::ParticipationObserved {
        peer_id: 3,
    }));

    // Assert
    assert_eq!(command, Command::ParticipationObserved { peer_id: 3 });
}

#[test]
#[should_panic(expected = "handled in decide()")]
fn to_command_timer_retry_membership_panics() {
    // Arrange & Act & Assert
    let _ = translator().to_command(InputMessage::Timer(TimerMessage::RetryMembership {
        peer_id: 5,
    }));
}

#[test]
#[should_panic(expected = "handled in decide()")]
fn to_command_timer_retry_ping_panics() {
    // Arrange
    let translator = translator();

    // Act & Assert
    let _ = translator.to_command(InputMessage::Timer(TimerMessage::RetryPing));
}

#[test]
#[should_panic(expected = "handled in decide()")]
fn to_command_timer_retry_ready_panics() {
    // Arrange
    let translator = translator();

    // Act & Assert
    let _ = translator.to_command(InputMessage::Timer(TimerMessage::RetryReady));
}

#[test]
fn to_command_transport_bootstrapped_maps_to_probe() {
    // Arrange
    let translator = translator();

    // Act
    let command = translator.to_command(InputMessage::Transport(TransportMessage::Bootstrapped {
        from: 1,
    }));

    // Assert
    assert_eq!(command, Command::Probe);
}

#[test]
fn to_command_transport_membership_acknowledged_maps_to_membership_acknowledged() {
    // Arrange
    let translator = translator();

    // Act
    let command = translator.to_command(InputMessage::Transport(
        TransportMessage::MembershipAcknowledged { from: 5 },
    ));

    // Assert
    assert_eq!(command, Command::MembershipAcknowledged { peer_id: 5 });
}

#[test]
#[should_panic(expected = "handled in decide()")]
fn to_command_transport_membership_panics() {
    // Arrange & Act & Assert
    let _ = translator().to_command(InputMessage::Transport(TransportMessage::Membership {
        from: 0,
        members: vec![0, 1],
        required_count: 2,
    }));
}

#[test]
fn to_command_transport_ping_maps_to_participation_observed() {
    // Arrange
    let translator = translator();

    // Act
    let command =
        translator.to_command(InputMessage::Transport(TransportMessage::Ping { from: 42 }));

    // Assert
    assert_eq!(command, Command::ParticipationObserved { peer_id: 42 });
}

#[test]
fn to_command_transport_ready_maps_to_ready_observed() {
    // Arrange
    let translator = translator();

    // Act
    let command =
        translator.to_command(InputMessage::Transport(TransportMessage::Ready { from: 7 }));

    // Assert
    assert_eq!(command, Command::ReadyObserved { peer_id: 7 });
}

#[test]
fn to_output_messages_addition_withdrawn_cancels_the_retry() {
    // Arrange
    let translator = translator();

    // Act
    let result =
        translator.to_output_messages(vec![Outcome::AdditionWithdrawn { peer_id: 5 }], &view(), 0);

    // Assert
    assert_eq!(
        result,
        vec![OutputMessage::Cancel(TimerEvent::Fire(
            TimerMessage::RetryMembership { peer_id: 5 }
        ))]
    );
}

#[test]
fn to_output_messages_all_non_matching_returns_noop() {
    // Arrange
    let translator = translator();

    // Act
    let result = translator.to_output_messages(
        vec![
            Outcome::ParticipationAccepted { peer_id: 0 },
            Outcome::ReadyAccepted { peer_id: 1 },
            Outcome::DuplicateParticipationIgnored { peer_id: 0 },
            Outcome::NonMemberIgnored { peer_id: 99 },
            Outcome::LocalParticipationCompleted,
        ],
        &view(),
        0,
    );

    // Assert
    assert_eq!(result, vec![OutputMessage::Noop]);
}

#[test]
fn to_output_messages_broadcast_local_ready_returns_broadcast_and_retry() {
    // Arrange
    let translator = translator();

    // Act
    let result = translator.to_output_messages(vec![Outcome::BroadcastLocalReady], &view(), 0);

    // Assert
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], OutputMessage::BroadcastReady));
    assert!(matches!(
        result[1],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady))
    ));
}

#[test]
fn to_output_messages_broadcast_local_ready_then_concluded_returns_both_in_order() {
    // Arrange
    let translator = translator();

    // Act
    let result = translator.to_output_messages(
        vec![
            Outcome::BroadcastLocalReady,
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ],
        &view(),
        0,
    );

    // Assert
    assert_eq!(
        result,
        vec![
            OutputMessage::BroadcastReady,
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::LocalParticipationCompleted)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryPing)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryReady)),
        ]
    );
}

#[test]
fn to_output_messages_concluded_then_broadcast_local_ready_returns_both_in_order() {
    // Arrange
    let translator = translator();

    // Act
    let result = translator.to_output_messages(
        vec![
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
            Outcome::BroadcastLocalReady,
        ],
        &view(),
        0,
    );

    // Assert
    assert_eq!(
        result,
        vec![
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::LocalParticipationCompleted)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryPing)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryReady)),
            OutputMessage::BroadcastReady,
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady)),
        ]
    );
}

#[test]
fn to_output_messages_empty_outcomes_returns_noop() {
    // Arrange
    let translator = translator();

    // Act
    let result = translator.to_output_messages(vec![], &view(), 0);

    // Assert
    assert_eq!(result, vec![OutputMessage::Noop]);
}

#[test]
fn to_output_messages_exited_returns_cancel_lpc_retry_ping_and_retry_ready() {
    // Arrange
    let translator = translator();

    // Act
    let result = translator.to_output_messages(
        vec![Outcome::Concluded {
            mode: Conclusion::Bootstrapped,
        }],
        &view(),
        0,
    );

    // Assert
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], OutputMessage::Cancel(_)));
    assert!(matches!(result[1], OutputMessage::Cancel(_)));
    assert!(matches!(result[2], OutputMessage::Cancel(_)));
}

#[test]
fn to_output_messages_of_a_completion_that_concludes_announces_readiness_and_stops_retrying() {
    // Arrange
    let translator = translator();

    // Act
    let result = translator.to_output_messages(
        vec![
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ],
        &view(),
        0,
    );

    // Assert
    assert_eq!(
        result,
        vec![
            OutputMessage::BroadcastReady,
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::LocalParticipationCompleted)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryPing)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryReady)),
        ]
    );
}

#[test]
fn to_output_messages_propose_addition_proposes_it_and_cancels_the_retry() {
    // Arrange
    let translator = translator();

    // Act
    let result =
        translator.to_output_messages(vec![Outcome::ProposeAddition { peer_id: 5 }], &view(), 0);

    // Assert
    assert_eq!(
        result,
        vec![
            OutputMessage::ProposeAddition { peer_id: 5 },
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryMembership {
                peer_id: 5
            })),
        ]
    );
}

#[test]
fn to_output_messages_send_membership_sends_the_view_s_membership_and_schedules_its_retry() {
    // Arrange
    let translator = translator();

    // Act
    let result =
        translator.to_output_messages(vec![Outcome::SendMembership { peer_id: 5 }], &view(), 0);

    // Assert
    assert_eq!(
        result,
        vec![
            OutputMessage::Send {
                to: 5,
                message: TransportMessage::Membership {
                    from: 0,
                    members: vec![0, 1, 2],
                    required_count: 2,
                },
            },
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryMembership {
                peer_id: 5
            })),
        ]
    );
}

#[test]
fn to_output_messages_skips_non_matching_and_finds_broadcast_local_ready() {
    // Arrange
    let translator = translator();

    // Act
    let result = translator.to_output_messages(
        vec![
            Outcome::ParticipationAccepted { peer_id: 1 },
            Outcome::LocalParticipationCompleted,
            Outcome::BroadcastLocalReady,
        ],
        &view(),
        0,
    );

    // Assert
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], OutputMessage::BroadcastReady));
}

#[test]
fn to_output_messages_skips_non_matching_and_finds_exited() {
    // Arrange
    let translator = translator();

    // Act
    let result = translator.to_output_messages(
        vec![
            Outcome::ReadyAccepted { peer_id: 2 },
            Outcome::Concluded {
                mode: Conclusion::Bootstrapped,
            },
        ],
        &view(),
        0,
    );

    // Assert
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], OutputMessage::Cancel(_)));
}
