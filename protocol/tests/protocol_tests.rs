// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

fn protocol() -> Protocol {
    let config = Config::new(0, vec![0, 1], QuorumPolicy::new(2));
    Protocol::new(Faction::new(config, Box::new(NoOpObserver)), vec![0, 1], 0)
}

#[test]
fn initialize_with_two_peers_produces_decisions() {
    // Arrange & Act
    let decisions = protocol().initialize();

    // Assert
    assert_eq!(decisions.len(), 4);
    assert!(matches!(
        decisions[0],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ParticipationObserved {
            peer_id: 1
        }))
    ));
    assert!(matches!(
        decisions[1],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::LocalParticipationCompleted))
    ));
    assert!(matches!(decisions[2], OutputMessage::BroadcastPing));
    assert!(matches!(
        decisions[3],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryPing))
    ));
}

#[test]
fn decide_ping_from_member_produces_broadcast() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let decisions = protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::Pinging);
}

#[test]
fn decide_ping_from_non_member_produces_noop_with_state_transition() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let decisions = protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 99 }));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::Pinging);
}

#[test]
fn decide_local_completion_produces_broadcast_ready_and_retry() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));

    // Act
    let decisions = protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));

    // Assert
    assert_eq!(decisions.len(), 2);
    assert!(matches!(decisions[0], OutputMessage::BroadcastReady));
    assert!(matches!(
        decisions[1],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady))
    ));
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::Collecting);
}

#[test]
fn decide_ready_after_local_completion_reaches_quorum() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));

    // Act
    let decisions = protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));

    // Assert
    assert_eq!(decisions.len(), 3);
    assert!(matches!(decisions[0], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[1], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[2], OutputMessage::Cancel(_)));
    assert_eq!(
        protocol.cluster_view().peer_state(),
        PeerState::Bootstrapped
    );
}

#[test]
fn decide_ready_before_local_completion_is_noop() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));

    // Act
    let decisions = protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
}

#[test]
fn decide_deadline_expired_exits() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::DeadlineExpired));

    // Assert
    assert_eq!(decisions.len(), 3);
    assert!(matches!(decisions[0], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[1], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[2], OutputMessage::Cancel(_)));
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::TimedOut);
}

#[test]
fn decide_rejected_command_returns_noop() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));

    // Act — LocalParticipationCompleted is rejected in Collecting
    let decisions = protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
}

#[test]
fn decide_after_bootstrapped_rejects_all() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));
    protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));

    // Act
    let decisions = protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 0 }));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
}

#[test]
fn decide_retry_ready_while_active_produces_broadcast_and_retry() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::RetryReady));

    // Assert
    assert_eq!(decisions.len(), 2);
    assert!(matches!(decisions[0], OutputMessage::BroadcastReady));
    assert!(matches!(
        decisions[1],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady))
    ));
}

#[test]
fn decide_retry_ready_while_exited_produces_noop() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));
    protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::RetryReady));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
}

#[test]
fn decide_retry_ping_while_active_produces_broadcast_and_retry() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::RetryPing));

    // Assert
    assert_eq!(decisions.len(), 2);
    assert!(matches!(decisions[0], OutputMessage::BroadcastPing));
    assert!(matches!(
        decisions[1],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryPing))
    ));
}

#[test]
fn decide_retry_ping_while_exited_produces_noop() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));
    protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::RetryPing));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
}

#[test]
fn decide_timer_participation_observed_produces_noop() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::ParticipationObserved {
        peer_id: 1,
    }));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::Pinging);
}

#[test]
fn decide_bootstrapped_message_produces_noop() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let decisions = protocol.decide(InputMessage::Transport(TransportMessage::Bootstrapped {
        from: 1,
    }));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
}

#[test]
fn decide_convergence_sequence_reaches_bootstrapped() {
    // Arrange
    let mut protocol = protocol();

    // Act — Ping
    let decisions = protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));

    // Assert
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::Pinging);

    // Act — LocalParticipationCompleted
    let decisions = protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));

    // Assert
    assert_eq!(decisions.len(), 2);
    assert!(matches!(decisions[0], OutputMessage::BroadcastReady));
    assert!(matches!(
        decisions[1],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady))
    ));
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::Collecting);

    // Act — Ready from peer
    let decisions = protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));

    // Assert
    assert_eq!(decisions.len(), 3);
    assert!(matches!(decisions[0], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[1], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[2], OutputMessage::Cancel(_)));
    assert_eq!(
        protocol.cluster_view().peer_state(),
        PeerState::Bootstrapped
    );
}

#[test]
fn decide_pre_accumulated_ready_quorum_on_completion() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));

    // Act — LPC fires with Ready already accumulated
    let decisions = protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));

    // Assert
    assert_eq!(decisions.len(), 2);
    assert!(matches!(decisions[0], OutputMessage::BroadcastReady));
    assert!(matches!(
        decisions[1],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady))
    ));
    assert_eq!(
        protocol.cluster_view().peer_state(),
        PeerState::Bootstrapped
    );
}
