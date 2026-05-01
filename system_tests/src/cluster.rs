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

use crate::timer::in_memory::InMemoryTimer;
use crate::timer::timer_trait::Timer;
use crate::transport::in_memory::InMemoryTransport;
use crate::transport::transport_trait::Transport;

pub struct Cluster {
    peer_ids: Vec<PeerId>,
    protocols: Vec<Protocol>,
    timers: Vec<InMemoryTimer>,
    transports: Vec<InMemoryTransport>,
    drops: Vec<Drop>,
}

struct Drop {
    from: PeerId,
    to: PeerId,
    message: TransportMessage,
    remaining: usize,
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

        let timers = (0..count).map(|_| InMemoryTimer::new()).collect();
        let transports = InMemoryTransport::new_mesh(&peer_ids);

        Self {
            peer_ids,
            protocols,
            timers,
            transports,
            drops: Vec::new(),
        }
    }

    pub fn drop_message(
        &mut self,
        from: PeerId,
        to: PeerId,
        message: TransportMessage,
        count: usize,
    ) {
        self.drops.push(Drop {
            from,
            to,
            message,
            remaining: count,
        });
    }

    pub fn start_all(&mut self) {
        for i in 0..self.peer_ids.len() {
            for decision in self.protocols[i].start_decisions() {
                self.route(decision, self.peer_ids[i]);
            }
        }
    }

    pub fn step_transport(&mut self) -> bool {
        let mut any = false;

        for i in 0..self.peer_ids.len() {
            if let Some((from, msg)) = self.transports[i].recv() {
                any = true;
                for decision in self.protocols[i].decide(InputMessage::Transport(msg)) {
                    self.route(decision, from);
                }
            }
        }

        any
    }

    pub fn step_timer(&mut self) -> bool {
        let mut any = false;

        for i in 0..self.peer_ids.len() {
            if let Some(event) = self.timers[i].poll() {
                any = true;
                let TimerEvent::Fire(tm) = event;
                for decision in self.protocols[i].decide(InputMessage::Timer(tm)) {
                    self.route(decision, self.peer_ids[i]);
                }
            }
        }

        any
    }

    fn route(&mut self, msg: OutputMessage, from: PeerId) {
        let index = self.peer_ids.iter().position(|&p| p == from).unwrap();

        match msg {
            OutputMessage::BroadcastReady => {
                let peers = self.peer_ids.clone();
                for &to in &peers {
                    if to != from {
                        let transport_msg = TransportMessage::Ready { from };
                        if self.should_drop(from, to, &transport_msg) {
                            continue;
                        }
                        self.transports[index].send(to, transport_msg);
                    }
                }
            }
            OutputMessage::Schedule(event) => {
                self.timers[index].schedule(event);
            }
            OutputMessage::Cancel(event) => {
                self.timers[index].cancel(event);
            }
            OutputMessage::Noop => {}
        }
    }

    fn should_drop(&mut self, from: PeerId, to: PeerId, msg: &TransportMessage) -> bool {
        for drop in &mut self.drops {
            if drop.from == from && drop.to == to && drop.message == *msg && drop.remaining > 0 {
                drop.remaining -= 1;
                return true;
            }
        }
        false
    }

    pub fn node_state(&mut self, index: usize) -> PeerState {
        self.protocols[index].cluster_view().peer_state()
    }

    pub fn is_bootstrapped(&mut self) -> bool {
        self.protocols
            .iter_mut()
            .all(|p| p.cluster_view().peer_state() == PeerState::Bootstrapped)
    }
}
