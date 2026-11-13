// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;
use faction::reconfiguration::Reconfiguration;
use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_protocol::transport_message::TransportMessage;

// Node 0 of members 0 and 1, requiring two, concluded.
fn bootstrapped() -> Protocol {
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));
    protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));
    protocol
}

fn membership_for(peer_id: u64) -> OutputMessage {
    OutputMessage::Send {
        to: peer_id,
        message: TransportMessage::Membership {
            from: 0,
            members: vec![0, 1],
            required_count: 2,
        },
    }
}

// Node 0 of members 0 to 3, requiring one, concluded by its own completion,
// asking one helper.
fn of_four() -> Protocol {
    let config = Config::new(0, vec![0, 1, 2, 3], QuorumPolicy::new(1), 1);
    let mut protocol = Protocol::new(
        Faction::new(config, Box::new(NoOpObserver)),
        vec![0, 1, 2, 3],
        0,
        1,
    );
    protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));
    protocol
}

fn protocol() -> Protocol {
    let config = Config::new(0, vec![0, 1], QuorumPolicy::new(2), 1);
    Protocol::new(
        Faction::new(config, Box::new(NoOpObserver)),
        vec![0, 1],
        0,
        2,
    )
}

fn retry_membership(peer_id: u64) -> TimerEvent {
    TimerEvent::Fire(TimerMessage::RetryMembership { peer_id })
}

fn timeouts_cancelled(peer_id: u64) -> Vec<OutputMessage> {
    vec![
        OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::ProbeMissed { peer_id })),
        OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::IndirectProbesMissed {
            peer_id,
        })),
    ]
}

#[test]
fn abort_addition_of_the_newcomer_withdraws_it_and_cancels_the_retry() {
    // Arrange
    let mut protocol = bootstrapped();
    protocol.request_addition(5);

    // Act
    let decisions = protocol.abort_addition(5);

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Cancel(retry_membership(5))]);
    assert_eq!(
        protocol.cluster_view().reconfiguration(),
        Reconfiguration::None
    );
}

#[test]
fn abort_removal_of_the_peer_in_flight_withdraws_it() {
    // Arrange
    let mut protocol = of_four();
    protocol.request_removal(2);

    // Act
    let decisions = protocol.abort_removal(2);

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
    assert_eq!(
        protocol.cluster_view().reconfiguration(),
        Reconfiguration::None
    );
    assert_eq!(protocol.cluster_view().members().as_slice(), &[0, 1, 2, 3]);
}

#[test]
fn admit_new_peer_adds_it_to_members() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let decisions = protocol.admit(5);

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
    assert_eq!(protocol.cluster_view().members().as_slice(), &[0, 1, 5]);
}

#[test]
fn change_required_count_sets_the_count_in_effect() {
    // Arrange
    let mut protocol = bootstrapped();

    // Act
    let decisions = protocol.change_required_count(3);

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
    assert_eq!(protocol.cluster_view().required_count(), 3);
}

#[test]
fn commit_addition_of_the_newcomer_admits_it() {
    // Arrange
    let mut protocol = bootstrapped();
    protocol.request_addition(5);

    // Act
    let decisions = protocol.commit_addition(5);

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
    assert_eq!(protocol.cluster_view().members().as_slice(), &[0, 1, 5]);
    assert_eq!(
        protocol.cluster_view().reconfiguration(),
        Reconfiguration::None
    );
}

#[test]
fn commit_removal_of_the_node_itself_leaves_it_removed() {
    // Arrange
    let mut protocol = of_four();

    // Act
    let _ = protocol.commit_removal(0);

    // Assert
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::Removed);
    assert_eq!(protocol.cluster_view().members().as_slice(), &[1, 2, 3]);
}

#[test]
fn commit_removal_of_the_peer_in_flight_takes_it_out_and_cancels_its_timeouts() {
    // Arrange
    let mut protocol = of_four();
    protocol.request_removal(2);

    // Act
    let decisions = protocol.commit_removal(2);

    // Assert
    assert_eq!(decisions, timeouts_cancelled(2));
    assert_eq!(protocol.cluster_view().members().as_slice(), &[0, 1, 3]);
    assert_eq!(
        protocol.cluster_view().reconfiguration(),
        Reconfiguration::None
    );
}

#[test]
fn decide_after_bootstrapped_re_advertises_on_ping_and_ignores_ready() {
    // Arrange — drive the node to Bootstrapped (quorum 2: self plus peer 1)
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    protocol.decide(InputMessage::Timer(
        TimerMessage::LocalParticipationCompleted,
    ));
    protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));

    // Act & Assert — a member's ping re-advertises this node's readiness
    let on_ping = protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));
    assert_eq!(on_ping.len(), 1);
    assert!(matches!(on_ping[0], OutputMessage::BroadcastReady));

    // Act & Assert — a duplicate ready draws no reply
    let on_ready = protocol.decide(InputMessage::Transport(TransportMessage::Ready { from: 1 }));
    assert_eq!(on_ready.len(), 1);
    assert!(matches!(on_ready[0], OutputMessage::Noop));
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
    assert_eq!(decisions.len(), 4);
    assert!(matches!(decisions[0], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[1], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[2], OutputMessage::Cancel(_)));
    assert_eq!(
        decisions[3],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ProbeDue))
    );
    assert_eq!(
        protocol.cluster_view().peer_state(),
        PeerState::Bootstrapped
    );
}

#[test]
fn decide_deadline_expired_stays_receptive() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::DeadlineExpired));

    // Assert — a missed deadline is recorded, not concluded; retries keep running
    assert_eq!(decisions.len(), 1);
    assert!(matches!(decisions[0], OutputMessage::Noop));
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::TimedOut);
    assert_ne!(
        protocol.cluster_view().peer_state(),
        PeerState::Bootstrapped
    );
}

#[test]
fn decide_indirect_probe_answer_notes_the_target_alive_and_cancels_its_timeouts() {
    // Arrange
    let mut protocol = of_four();

    // Act
    let outputs = protocol.decide(InputMessage::Transport(
        TransportMessage::IndirectProbeAnswer { from: 2, target: 1 },
    ));

    // Assert
    assert_eq!(outputs, timeouts_cancelled(1));
}

#[test]
fn decide_indirect_probe_probes_the_target_for_the_node_that_asked() {
    // Arrange
    let mut protocol = of_four();

    // Act
    let outputs = protocol.decide(InputMessage::Transport(TransportMessage::IndirectProbe {
        from: 2,
        target: 1,
    }));

    // Assert
    assert_eq!(
        outputs,
        vec![OutputMessage::Send {
            to: 1,
            message: TransportMessage::Probe {
                from: 0,
                relay_for: Some(2),
            },
        }]
    );
}

#[test]
fn decide_indirect_probes_missed_at_the_threshold_suspects_the_member() {
    // Arrange
    let mut protocol = of_four();

    // Act
    let outputs = protocol.decide(InputMessage::Timer(TimerMessage::IndirectProbesMissed {
        peer_id: 1,
    }));

    // Assert
    assert_eq!(outputs, vec![OutputMessage::Noop]);
    assert_eq!(protocol.cluster_view().suspected(), &[1]);
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
fn decide_membership_acknowledged_by_the_newcomer_proposes_the_addition() {
    // Arrange
    let mut protocol = bootstrapped();
    protocol.request_addition(5);

    // Act
    let decisions = protocol.decide(InputMessage::Transport(
        TransportMessage::MembershipAcknowledged { from: 5 },
    ));

    // Assert
    assert_eq!(
        decisions,
        vec![
            OutputMessage::ProposeAddition { peer_id: 5 },
            OutputMessage::Cancel(retry_membership(5)),
        ]
    );
}

#[test]
fn decide_membership_at_a_running_node_acknowledges_it() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let decisions = protocol.decide(InputMessage::Transport(TransportMessage::Membership {
        from: 7,
        members: vec![7],
        required_count: 1,
    }));

    // Assert
    assert_eq!(
        decisions,
        vec![OutputMessage::Send {
            to: 7,
            message: TransportMessage::MembershipAcknowledged { from: 0 },
        }]
    );
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
    assert_eq!(
        decisions,
        vec![
            OutputMessage::BroadcastReady,
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::LocalParticipationCompleted)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryPing)),
            OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryReady)),
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ProbeDue)),
        ]
    );
    assert_eq!(
        protocol.cluster_view().peer_state(),
        PeerState::Bootstrapped
    );
}

#[test]
fn decide_probe_answer_notes_the_member_alive_and_cancels_its_timeouts() {
    // Arrange
    let mut protocol = of_four();

    // Act
    let outputs = protocol.decide(InputMessage::Transport(TransportMessage::ProbeAnswer {
        from: 1,
        relay_for: None,
    }));

    // Assert
    assert_eq!(outputs, timeouts_cancelled(1));
}

#[test]
fn decide_probe_due_before_the_conclusion_produces_noop() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let outputs = protocol.decide(InputMessage::Timer(TimerMessage::ProbeDue));

    // Assert
    assert_eq!(outputs, vec![OutputMessage::Noop]);
}

#[test]
fn decide_probe_due_once_removed_produces_noop_and_no_next_period() {
    // Arrange
    let mut protocol = of_four();
    let _ = protocol.commit_removal(0);

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::ProbeDue));

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
}

#[test]
fn decide_probe_due_probes_the_next_member_schedules_its_timeout_and_the_next_period() {
    // Arrange
    let mut protocol = of_four();

    // Act
    let outputs = protocol.decide(InputMessage::Timer(TimerMessage::ProbeDue));

    // Assert
    assert_eq!(
        outputs,
        vec![
            OutputMessage::Send {
                to: 1,
                message: TransportMessage::Probe {
                    from: 0,
                    relay_for: None,
                },
            },
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ProbeMissed { peer_id: 1 })),
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ProbeDue)),
        ]
    );
}

#[test]
fn decide_probe_from_a_peer_is_answered() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let outputs = protocol.decide(InputMessage::Transport(TransportMessage::Probe {
        from: 1,
        relay_for: None,
    }));

    // Assert
    assert_eq!(
        outputs,
        vec![OutputMessage::Send {
            to: 1,
            message: TransportMessage::ProbeAnswer {
                from: 0,
                relay_for: None,
            },
        }]
    );
}

#[test]
fn decide_probe_missed_asks_as_many_helpers_as_the_protocol_takes() {
    // Arrange
    let mut protocol = of_four();

    // Act
    let outputs = protocol.decide(InputMessage::Timer(TimerMessage::ProbeMissed {
        peer_id: 1,
    }));

    // Assert
    assert_eq!(
        outputs,
        vec![
            OutputMessage::Send {
                to: 2,
                message: TransportMessage::IndirectProbe { from: 0, target: 1 },
            },
            OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::IndirectProbesMissed {
                peer_id: 1
            })),
        ]
    );
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
    assert_eq!(decisions.len(), 4);
    assert!(matches!(decisions[0], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[1], OutputMessage::Cancel(_)));
    assert!(matches!(decisions[2], OutputMessage::Cancel(_)));
    assert_eq!(
        decisions[3],
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ProbeDue))
    );
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
fn decide_relayed_probe_answer_is_forwarded_to_the_node_that_asked() {
    // Arrange
    let mut protocol = of_four();

    // Act
    let outputs = protocol.decide(InputMessage::Transport(TransportMessage::ProbeAnswer {
        from: 1,
        relay_for: Some(2),
    }));

    // Assert
    assert_eq!(
        outputs,
        vec![OutputMessage::Send {
            to: 2,
            message: TransportMessage::IndirectProbeAnswer { from: 0, target: 1 },
        }]
    );
}

#[test]
fn decide_retry_membership_once_acknowledged_produces_noop() {
    // Arrange
    let mut protocol = bootstrapped();
    protocol.request_addition(5);
    protocol.decide(InputMessage::Transport(
        TransportMessage::MembershipAcknowledged { from: 5 },
    ));

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::RetryMembership {
        peer_id: 5,
    }));

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
}

#[test]
fn decide_retry_membership_while_requested_resends_and_reschedules() {
    // Arrange
    let mut protocol = bootstrapped();
    protocol.request_addition(5);

    // Act
    let decisions = protocol.decide(InputMessage::Timer(TimerMessage::RetryMembership {
        peer_id: 5,
    }));

    // Assert
    assert_eq!(
        decisions,
        vec![
            membership_for(5),
            OutputMessage::Schedule(retry_membership(5))
        ]
    );
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
fn deny_peer_leaves_members_unchanged() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let decisions = protocol.deny(5);

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
    assert_eq!(protocol.cluster_view().members().as_slice(), &[0, 1]);
}

#[test]
fn expire_deadline_while_fresh_is_rejected_as_noop() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let decisions = protocol.expire_deadline();

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::Fresh);
}

#[test]
fn expire_deadline_while_pinging_reports_timed_out() {
    // Arrange
    let mut protocol = protocol();
    protocol.decide(InputMessage::Transport(TransportMessage::Ping { from: 1 }));

    // Act
    let decisions = protocol.expire_deadline();

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
    assert_eq!(protocol.cluster_view().peer_state(), PeerState::TimedOut);
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
fn request_addition_after_a_removal_sends_the_membership_without_the_removed_member() {
    // Arrange
    let mut protocol = of_four();
    protocol.request_removal(2);
    let _ = protocol.commit_removal(2);

    // Act
    let decisions = protocol.request_addition(5);

    // Assert
    assert_eq!(
        decisions[0],
        OutputMessage::Send {
            to: 5,
            message: TransportMessage::Membership {
                from: 0,
                members: vec![0, 1, 3],
                required_count: 1,
            },
        }
    );
}

#[test]
fn request_addition_of_a_newcomer_sends_it_the_membership_and_schedules_the_retry() {
    // Arrange
    let mut protocol = bootstrapped();

    // Act
    let decisions = protocol.request_addition(5);

    // Assert
    assert_eq!(
        decisions,
        vec![
            membership_for(5),
            OutputMessage::Schedule(retry_membership(5))
        ]
    );
    assert_eq!(
        protocol.cluster_view().reconfiguration(),
        Reconfiguration::AdditionRequested { peer_id: 5 }
    );
}

#[test]
fn request_join_leaves_members_unchanged() {
    // Arrange
    let mut protocol = protocol();

    // Act
    let decisions = protocol.request_join(5);

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
    assert_eq!(protocol.cluster_view().members().as_slice(), &[0, 1]);
}

#[test]
fn request_removal_of_a_member_the_quorum_can_spare_proposes_it() {
    // Arrange
    let mut protocol = of_four();

    // Act
    let decisions = protocol.request_removal(2);

    // Assert
    assert_eq!(
        decisions,
        vec![OutputMessage::ProposeRemoval { peer_id: 2 }]
    );
    assert_eq!(
        protocol.cluster_view().reconfiguration(),
        Reconfiguration::RemovalRequested { peer_id: 2 }
    );
}

#[test]
fn request_removal_the_quorum_cannot_spare_is_refused_asking_nothing() {
    // Arrange
    let mut protocol = bootstrapped();

    // Act
    let decisions = protocol.request_removal(1);

    // Assert
    assert_eq!(decisions, vec![OutputMessage::Noop]);
    assert_eq!(
        protocol.cluster_view().reconfiguration(),
        Reconfiguration::None
    );
}
