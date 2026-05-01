// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::Freshness;
use faction::PeerId;
use faction::command::Command;

use crate::protocol::Decision;
use crate::protocol::Protocol;
use crate::timer::Timer;
use crate::timer::TimerEvent;
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
            self.dispatch(decision, 0);
        }
    }

    pub fn step(&mut self, current_marker: Freshness) {
        let command = if self.use_timer {
            match self.timer.poll() {
                Some(TimerEvent::Fire(command)) => command,
                None => {
                    self.use_timer = !self.use_timer;
                    return;
                }
            }
        } else {
            match self.transport.recv() {
                Some((_, command)) => command,
                None => {
                    self.use_timer = !self.use_timer;
                    return;
                }
            }
        };

        let decision = self.protocol.decide(command);
        self.dispatch(decision, current_marker);
        self.use_timer = !self.use_timer;
    }

    fn dispatch(&mut self, decision: Decision, current_marker: Freshness) {
        match decision {
            Decision::BroadcastReady => {
                for to in &self.peers {
                    if *to != self.peer_id {
                        self.transport.send(
                            *to,
                            Command::ReadyObserved {
                                peer_id: self.peer_id,
                                freshness: current_marker,
                                current_marker,
                            },
                        );
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
