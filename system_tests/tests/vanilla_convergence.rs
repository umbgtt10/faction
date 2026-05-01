// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::transport_message::TransportMessage;

use faction_system_tests::transport::in_memory::InMemoryTransport;
use faction_system_tests::transport::transport_trait::Transport;

#[test]
fn two_nodes_converge_to_bootstrapped() {
    // Arrange
    let config_a = Config::new(0, vec![0, 1], QuorumPolicy::new(2), FreshnessPolicy::new(2));
    let config_b = Config::new(1, vec![0, 1], QuorumPolicy::new(2), FreshnessPolicy::new(2));

    let mut protocol_a = Protocol::new(
        Faction::new(config_a, Box::new(NoOpObserver)),
        vec![0, 1],
        0,
    );
    let mut protocol_b = Protocol::new(
        Faction::new(config_b, Box::new(NoOpObserver)),
        vec![0, 1],
        1,
    );

    let (mut transport_a, mut transport_b) = InMemoryTransport::new_pair(0, 1);

    // Act — node A fires start_decisions, which schedules ParticipationObserved and
    // LocalParticipationCompleted for peer B. Node A processes them and broadcasts Ready.
    for decision in protocol_a.start_decisions() {
        for sub in dispatch_immediate(decision, &mut transport_a, 0) {
            for msg in protocol_a.decide(sub) {
                dispatch(msg, &mut transport_a, 0);
            }
        }
    }

    // Drain any Ready messages from node A to node B
    while let Some((from, msg)) = transport_b.recv() {
        for decision in protocol_b.decide(InputMessage::Transport(msg)) {
            dispatch(decision, &mut transport_b, from);
        }
    }

    // Node B fires its own start_decisions
    for decision in protocol_b.start_decisions() {
        for sub in dispatch_immediate(decision, &mut transport_b, 1) {
            for msg in protocol_b.decide(sub) {
                dispatch(msg, &mut transport_b, 1);
            }
        }
    }

    // Drain messages back to node A
    while let Some((from, msg)) = transport_a.recv() {
        for decision in protocol_a.decide(InputMessage::Transport(msg)) {
            dispatch(decision, &mut transport_a, from);
        }
    }

    // Assert
    assert_eq!(
        protocol_a.cluster_view().peer_state(),
        PeerState::Bootstrapped
    );
    assert_eq!(
        protocol_b.cluster_view().peer_state(),
        PeerState::Bootstrapped
    );
}

fn dispatch(msg: OutputMessage, transport: &mut InMemoryTransport, from: PeerId) {
    match msg {
        OutputMessage::BroadcastReady => {
            transport.send(1 - from, TransportMessage::Ready { from });
        }
        OutputMessage::Noop => {}
        _ => {}
    }
}

fn dispatch_immediate(
    msg: OutputMessage,
    transport: &mut InMemoryTransport,
    from: PeerId,
) -> Vec<InputMessage> {
    match msg {
        OutputMessage::Schedule(event) => {
            let timer_msg = match event {
                TimerEvent::Fire(tm) => tm,
            };
            vec![InputMessage::Timer(timer_msg)]
        }
        _ => vec![],
    }
}
