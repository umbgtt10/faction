// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::thread::sleep;
use std::time::Duration;

use crate::node_observer::NodeObserver;
use faction::peer_state::PeerState;
use faction::types::PeerId;

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::transport_message::TransportMessage;

use faction_protocol::timer_trait::Timer;
use faction_protocol::transport_trait::Transport;

pub struct FactionNode {
    peer_id: PeerId,
    peers: Vec<PeerId>,
    protocol: Protocol,
    transport: Box<dyn Transport>,
    timer: Box<dyn Timer>,
    observer: Box<dyn NodeObserver>,
    toggle_timer_and_transport: bool,
    idle_delay: Duration,
}

impl FactionNode {
    pub fn new(
        peer_id: PeerId,
        peers: Vec<PeerId>,
        protocol: Protocol,
        transport: Box<dyn Transport>,
        timer: Box<dyn Timer>,
        observer: Box<dyn NodeObserver>,
        idle_delay: Duration,
    ) -> Self {
        Self {
            peer_id,
            peers,
            protocol,
            transport,
            timer,
            observer,
            toggle_timer_and_transport: true,
            idle_delay,
        }
    }

    pub fn start(&mut self) {
        let decisions = self.protocol.initialize();

        self.observer.on_start();
        for decision in decisions {
            self.dispatch(decision);
        }
    }

    #[must_use]
    pub fn peer_state(&mut self) -> PeerState {
        self.protocol.cluster_view().peer_state()
    }

    pub fn is_concluded(&mut self) -> bool {
        matches!(
            self.peer_state(),
            PeerState::Bootstrapped | PeerState::TimedOut
        )
    }

    pub fn run(&mut self) {
        self.start();
        loop {
            let had_work = self.step_internal();
            if self.is_concluded() {
                break;
            }
            if !had_work {
                sleep(self.idle_delay);
            }
        }
    }

    fn step_internal(&mut self) -> bool {
        let message = if self.toggle_timer_and_transport {
            match self.timer.poll() {
                Some(TimerEvent::Fire(timer_msg)) => InputMessage::Timer(timer_msg),
                None => {
                    self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
                    self.observer.on_idle();
                    return false;
                }
            }
        } else {
            match self.transport.recv() {
                Some(transport_msg) => InputMessage::Transport(transport_msg),
                None => {
                    self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
                    self.observer.on_idle();
                    return false;
                }
            }
        };

        let decisions = self.protocol.decide(message.clone());
        self.observer.on_step(&message, &decisions);
        for decision in decisions {
            self.dispatch(decision);
        }
        self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
        true
    }

    pub fn step(&mut self) {
        self.step_internal();
    }

    pub fn request_join(&mut self, peer_id: PeerId) {
        let _ = self.protocol.request_join(peer_id);
    }

    pub fn admit(&mut self, peer_id: PeerId) {
        let _ = self.protocol.admit(peer_id);
        if !self.peers.contains(&peer_id) {
            self.peers.push(peer_id);
        }
    }

    pub fn deny(&mut self, peer_id: PeerId) {
        let _ = self.protocol.deny(peer_id);
    }

    fn dispatch(&mut self, decision: OutputMessage) {
        match decision {
            OutputMessage::BroadcastPing => {
                for to in &self.peers {
                    if *to != self.peer_id {
                        self.transport
                            .send(*to, TransportMessage::Ping { from: self.peer_id });
                    }
                }
            }
            OutputMessage::BroadcastReady => {
                for to in &self.peers {
                    if *to != self.peer_id {
                        self.transport
                            .send(*to, TransportMessage::Ready { from: self.peer_id });
                    }
                }
            }
            OutputMessage::Schedule(event) => {
                self.timer.schedule(event);
            }
            OutputMessage::Cancel(event) => {
                self.timer.cancel(event);
            }
            OutputMessage::Noop => {}
        }
    }
}
