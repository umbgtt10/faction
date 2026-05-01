// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::node_observer::NodeObserver;
use faction::PeerId;
use faction::peer_state::PeerState;

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
}

impl FactionNode {
    pub fn new(
        peer_id: PeerId,
        peers: Vec<PeerId>,
        protocol: Protocol,
        transport: Box<dyn Transport>,
        timer: Box<dyn Timer>,
        observer: Box<dyn NodeObserver>,
    ) -> Self {
        Self {
            peer_id,
            peers,
            protocol,
            transport,
            timer,
            observer,
            toggle_timer_and_transport: true,
        }
    }

    pub fn start(&mut self) {
        let decisions = self.protocol.start_decisions();

        self.observer.on_start();
        for decision in decisions {
            self.dispatch(decision);
        }
    }

    #[must_use]
    pub fn peer_state(&mut self) -> PeerState {
        self.protocol.cluster_view().peer_state()
    }

    pub fn is_terminal(&mut self) -> bool {
        matches!(self.peer_state(), PeerState::Bootstrapped | PeerState::TimedOut)
    }

    pub fn run(&mut self) {
        self.start();
        while !self.is_terminal() {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let message = if self.toggle_timer_and_transport {
            match self.timer.poll() {
                Some(TimerEvent::Fire(timer_msg)) => InputMessage::Timer(timer_msg),
                None => {
                    self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
                    self.observer.on_idle();
                    return;
                }
            }
        } else {
            match self.transport.recv() {
                Some(transport_msg) => InputMessage::Transport(transport_msg),
                None => {
                    self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
                    self.observer.on_idle();
                    return;
                }
            }
        };

        let decisions = self.protocol.decide(message.clone());
        self.observer.on_step(&message, &decisions);
        for decision in decisions {
            self.dispatch(decision);
        }
        self.toggle_timer_and_transport = !self.toggle_timer_and_transport;
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
