// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;

use faction_protocol::message::Message;
use faction_protocol::message::TransportMessage;
use faction_protocol::protocol::Decision;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer_event::TimerEvent;

use crate::timer::Timer;
use crate::transport::transport_trait::Transport;

pub struct FactionNode {
    peer_id: PeerId,
    peers: Vec<PeerId>,
    protocol: Protocol,
    transport: Box<dyn Transport>,
    timer: Box<dyn Timer>,
    use_timer: bool,
}

impl FactionNode {
    pub fn new(
        peer_id: PeerId,
        peers: Vec<PeerId>,
        protocol: Protocol,
        transport: Box<dyn Transport>,
        timer: Box<dyn Timer>,
    ) -> Self {
        Self {
            peer_id,
            peers,
            protocol,
            transport,
            timer,
            use_timer: true,
        }
    }

    pub fn start(&mut self) {
        let decisions = self.protocol.start_decisions(&self.peers, self.peer_id);

        for decision in decisions {
            self.dispatch(decision);
        }
    }

    pub fn step(&mut self) {
        let message = if self.use_timer {
            match self.timer.poll() {
                Some(TimerEvent::Fire(timer_msg)) => Message::Timer(timer_msg),
                None => {
                    self.use_timer = !self.use_timer;
                    return;
                }
            }
        } else {
            match self.transport.recv() {
                Some((_, transport_msg)) => Message::Transport(transport_msg),
                None => {
                    self.use_timer = !self.use_timer;
                    return;
                }
            }
        };

        let decision = self.protocol.decide(message);
        self.dispatch(decision);
        self.use_timer = !self.use_timer;
    }

    fn dispatch(&mut self, decision: Decision) {
        match decision {
            Decision::BroadcastReady => {
                for to in &self.peers {
                    if *to != self.peer_id {
                        self.transport
                            .send(*to, TransportMessage::Ready { from: self.peer_id });
                    }
                }
            }
            Decision::Schedule(event) => {
                self.timer.schedule(event);
            }
            Decision::Cancel(event) => {
                self.timer.cancel(event);
            }
            Decision::Noop => {}
        }
    }
}
