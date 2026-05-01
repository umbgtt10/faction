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

pub struct Cluster {
    protocol_a: Protocol,
    protocol_b: Protocol,
    transport_a: InMemoryTransport,
    transport_b: InMemoryTransport,
}

impl Cluster {
    pub fn new() -> Self {
        let config_a = Config::new(0, vec![0, 1], QuorumPolicy::new(2), FreshnessPolicy::new(2));
        let config_b = Config::new(1, vec![0, 1], QuorumPolicy::new(2), FreshnessPolicy::new(2));

        let protocol_a = Protocol::new(
            Faction::new(config_a, Box::new(NoOpObserver)),
            vec![0, 1],
            0,
        );
        let protocol_b = Protocol::new(
            Faction::new(config_b, Box::new(NoOpObserver)),
            vec![0, 1],
            1,
        );

        let (transport_a, transport_b) = InMemoryTransport::new_pair(0, 1);

        Self {
            protocol_a,
            protocol_b,
            transport_a,
            transport_b,
        }
    }

    pub fn converge(&mut self) {
        self.start_node_a();
        self.drain_to_node_b();
        self.start_node_b();
        self.drain_to_node_a();
    }

    fn start_node_a(&mut self) {
        for decision in self.protocol_a.start_decisions() {
            for input in self.immediate(decision) {
                for msg in self.protocol_a.decide(input) {
                    self.route(msg, 0);
                }
            }
        }
    }

    fn start_node_b(&mut self) {
        for decision in self.protocol_b.start_decisions() {
            for input in self.immediate(decision) {
                for msg in self.protocol_b.decide(input) {
                    self.route(msg, 1);
                }
            }
        }
    }

    fn drain_to_node_b(&mut self) {
        while let Some((from, msg)) = self.transport_b.recv() {
            for decision in self.protocol_b.decide(InputMessage::Transport(msg)) {
                self.route(decision, from);
            }
        }
    }

    fn drain_to_node_a(&mut self) {
        while let Some((from, msg)) = self.transport_a.recv() {
            for decision in self.protocol_a.decide(InputMessage::Transport(msg)) {
                self.route(decision, from);
            }
        }
    }

    fn route(&mut self, msg: OutputMessage, from: PeerId) {
        match msg {
            OutputMessage::BroadcastReady => {
                self.transport(from)
                    .send(other(from), TransportMessage::Ready { from });
            }
            OutputMessage::Noop => {}
            _ => {}
        }
    }

    fn immediate(&mut self, msg: OutputMessage) -> Vec<InputMessage> {
        match msg {
            OutputMessage::Schedule(event) => {
                let TimerEvent::Fire(tm) = event;
                vec![InputMessage::Timer(tm)]
            }
            _ => vec![],
        }
    }

    fn transport(&mut self, from: PeerId) -> &mut InMemoryTransport {
        if from == 0 {
            &mut self.transport_a
        } else {
            &mut self.transport_b
        }
    }

    pub fn is_bootstrapped(&mut self) -> bool {
        self.protocol_a.cluster_view().peer_state() == PeerState::Bootstrapped
            && self.protocol_b.cluster_view().peer_state() == PeerState::Bootstrapped
    }
}

fn other(from: PeerId) -> PeerId {
    if from == 0 { 1 } else { 0 }
}
