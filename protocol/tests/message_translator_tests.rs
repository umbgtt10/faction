// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::command::Command;
use faction::exit_mode::ExitMode;
use faction::outcome::Outcome;

use faction_protocol::input_message::InputMessage;
use faction_protocol::message_translator::MessageTranslator;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

fn translator() -> MessageTranslator {
    MessageTranslator::new()
}

#[test]
fn to_command_transport_ping_maps_to_participation_observed() {
    // Arrange
    let t = translator();

    // Act
    let command = t.to_command(InputMessage::Transport(TransportMessage::Ping { from: 42 }));

    // Assert
    assert_eq!(
        command,
        Command::ParticipationObserved {
            peer_id: 42,
            freshness: 0,
            current_marker: 0,
        }
    );
}

#[test]
fn to_command_transport_ready_maps_to_ready_observed() {
    // Arrange
    let t = translator();

    // Act
    let command = t.to_command(InputMessage::Transport(TransportMessage::Ready { from: 7 }));

    // Assert
    assert_eq!(
        command,
        Command::ReadyObserved {
            peer_id: 7,
            freshness: 0,
            current_marker: 0,
        }
    );
}

#[test]
fn to_command_transport_bootstrapped_maps_to_probe() {
    // Arrange
    let t = translator();

    // Act
    let command = t.to_command(InputMessage::Transport(TransportMessage::Bootstrapped {
        from: 1,
    }));

    // Assert
    assert_eq!(command, Command::Probe);
}

#[test]
fn to_command_timer_participation_observed_maps_to_participation_observed() {
    // Arrange
    let t = translator();

    // Act
    let command = t.to_command(InputMessage::Timer(TimerMessage::ParticipationObserved {
        peer_id: 3,
    }));

    // Assert
    assert_eq!(
        command,
        Command::ParticipationObserved {
            peer_id: 3,
            freshness: 0,
            current_marker: 0,
        }
    );
}

#[test]
fn to_command_timer_local_participation_completed_maps_to_local_participation_completed() {
    // Arrange
    let t = translator();

    // Act
    let command = t.to_command(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));

    // Assert
    assert_eq!(command, Command::LocalParticipationCompleted);
}

#[test]
#[should_panic(expected = "handled in decide()")]
fn to_command_timer_retry_ping_panics() {
    // Arrange
    let t = translator();

    // Act
    let _ = t.to_command(InputMessage::Timer(TimerMessage::RetryPing));
}

#[test]
#[should_panic(expected = "handled in decide()")]
fn to_command_timer_retry_ready_panics() {
    // Arrange
    let t = translator();

    // Act
    let _ = t.to_command(InputMessage::Timer(TimerMessage::RetryReady));
}

#[test]
fn to_command_timer_deadline_expired_maps_to_deadline_expired() {
    // Arrange
    let t = translator();

    // Act
    let command = t.to_command(InputMessage::Timer(TimerMessage::DeadlineExpired));

    // Assert
    assert_eq!(command, Command::DeadlineExpired);
}

#[test]
fn to_output_messages_empty_outcomes_returns_noop() {
    // Arrange
    let t = translator();

    // Act
    let result = t.to_output_messages(vec![]);

    // Assert
    assert_eq!(result, vec![OutputMessage::Noop]);
}

#[test]
fn to_output_messages_broadcast_local_ready_returns_broadcast_and_retry() {
    // Arrange
    let t = translator();

    // Act
    let result = t.to_output_messages(vec![Outcome::BroadcastLocalReady]);

    // Assert
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], OutputMessage::BroadcastReady));
    assert!(matches!(
        result[1],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady))
    ));
}

#[test]
fn to_output_messages_exited_returns_cancel_lpc_retry_ping_and_retry_ready() {
    // Arrange
    let t = translator();

    // Act
    let result = t.to_output_messages(vec![Outcome::Exited {
        mode: ExitMode::Bootstrapped,
    }]);

    // Assert
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], OutputMessage::Cancel(_)));
    assert!(matches!(result[1], OutputMessage::Cancel(_)));
    assert!(matches!(result[2], OutputMessage::Cancel(_)));
}

#[test]
fn to_output_messages_broadcast_local_ready_wins_over_exited_when_first() {
    // Arrange
    let t = translator();

    // Act
    let result = t.to_output_messages(vec![
        Outcome::BroadcastLocalReady,
        Outcome::Exited {
            mode: ExitMode::Bootstrapped,
        },
    ]);

    // Assert
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], OutputMessage::BroadcastReady));
    assert!(matches!(
        result[1],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady))
    ));
}

#[test]
fn to_output_messages_exited_wins_over_broadcast_local_ready_when_first() {
    // Arrange
    let t = translator();

    // Act
    let result = t.to_output_messages(vec![
        Outcome::Exited {
            mode: ExitMode::TimedOut,
        },
        Outcome::BroadcastLocalReady,
    ]);

    // Assert
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], OutputMessage::Cancel(_)));
    assert!(matches!(result[1], OutputMessage::Cancel(_)));
    assert!(matches!(result[2], OutputMessage::Cancel(_)));
}

#[test]
fn to_output_messages_skips_non_matching_and_finds_broadcast_local_ready() {
    // Arrange
    let t = translator();

    // Act
    let result = t.to_output_messages(vec![
        Outcome::ParticipationAccepted { peer_id: 1 },
        Outcome::LocalParticipationCompleted,
        Outcome::BroadcastLocalReady,
    ]);

    // Assert
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], OutputMessage::BroadcastReady));
}

#[test]
fn to_output_messages_skips_non_matching_and_finds_exited() {
    // Arrange
    let t = translator();

    // Act
    let result = t.to_output_messages(vec![
        Outcome::ReadyAccepted { peer_id: 2 },
        Outcome::Exited {
            mode: ExitMode::Bootstrapped,
        },
    ]);

    // Assert
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], OutputMessage::Cancel(_)));
}

#[test]
fn to_output_messages_all_non_matching_returns_noop() {
    // Arrange
    let t = translator();

    // Act
    let result = t.to_output_messages(vec![
        Outcome::ParticipationAccepted { peer_id: 0 },
        Outcome::ReadyAccepted { peer_id: 1 },
        Outcome::DuplicateParticipationIgnored { peer_id: 0 },
        Outcome::StaleReadyIgnored { peer_id: 2 },
        Outcome::NonMemberIgnored { peer_id: 99 },
        Outcome::DelayedParticipationAccepted { peer_id: 3 },
        Outcome::DelayedReadyAccepted { peer_id: 4 },
        Outcome::LocalParticipationCompleted,
    ]);

    // Assert
    assert_eq!(result, vec![OutputMessage::Noop]);
}
