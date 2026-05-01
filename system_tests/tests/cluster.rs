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
    peer_ids: Vec<PeerId>,
    protocols: Vec<Protocol>,
    transports: Vec<InMemoryTransport>,
}

impl Cluster {
    pub fn new(count: usize, required: usize) -> Self {
        let peer_ids: Vec<PeerId> = (0..count as PeerId).collect();

        let protocols = peer_ids
            .iter()
            .map(|&id| {
                let config = Config::new(
                    id,
                    peer_ids.clone(),
                    QuorumPolicy::new(required),
                    FreshnessPolicy::new(2),
                );
                Protocol::new(
                    Faction::new(config, Box::new(NoOpObserver)),
                    peer_ids.clone(),
                    id,
                )
            })
            .collect();

        let transports = InMemoryTransport::new_mesh(&peer_ids);

        Self {
            peer_ids,
            protocols,
            transports,
        }
    }

    pub fn converge(&mut self) {
        for i in 0..self.peer_ids.len() {
            self.start_node(i);
            self.drain_all();
        }
    }

    fn start_node(&mut self, index: usize) {
        for decision in self.protocols[index].start_decisions() {
            for input in self.immediate(decision) {
                for msg in self.protocols[index].decide(input) {
                    self.route(msg, self.peer_ids[index]);
                }
            }
        }
    }

    fn drain_all(&mut self) {
        loop {
            let mut any = false;
            for i in 0..self.peer_ids.len() {
                while let Some((from, msg)) = self.transports[i].recv() {
                    any = true;
                    for decision in self.protocols[i].decide(InputMessage::Transport(msg)) {
                        self.route(decision, from);
                    }
                }
            }
            if !any {
                break;
            }
        }
    }

    fn route(&mut self, msg: OutputMessage, from: PeerId) {
        match msg {
            OutputMessage::BroadcastReady => {
                let peers = self.peer_ids.clone();
                for &to in &peers {
                    if to != from {
                        self.transport(from)
                            .send(to, TransportMessage::Ready { from });
                    }
                }
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

    fn transport(&mut self, peer_id: PeerId) -> &mut InMemoryTransport {
        let index = self.peer_ids.iter().position(|&p| p == peer_id).unwrap();
        &mut self.transports[index]
    }

    pub fn is_bootstrapped(&mut self) -> bool {
        self.protocols
            .iter_mut()
            .all(|p| p.cluster_view().peer_state() == PeerState::Bootstrapped)
    }
}
