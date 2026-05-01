// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::PeerId;

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::transport_message::TransportMessage;

use crate::timer::timer_trait::Timer;
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
        let decisions = self.protocol.start_decisions();

        for decision in decisions {
            self.dispatch(decision);
        }
    }

    pub fn step(&mut self) {
        let message = if self.use_timer {
            match self.timer.poll() {
                Some(TimerEvent::Fire(timer_msg)) => InputMessage::Timer(timer_msg),
                None => {
                    self.use_timer = !self.use_timer;
                    return;
                }
            }
        } else {
            match self.transport.recv() {
                Some((_, transport_msg)) => InputMessage::Transport(transport_msg),
                None => {
                    self.use_timer = !self.use_timer;
                    return;
                }
            }
        };

        let decisions = self.protocol.decide(message);
        for decision in decisions {
            self.dispatch(decision);
        }
        self.use_timer = !self.use_timer;
    }

    fn dispatch(&mut self, decision: OutputMessage) {
        match decision {
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
