// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::Freshness;
use faction::PeerId;
use faction::command::Command;
use faction::outcome::Outcome;

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
        }
    }

    pub fn start(&mut self) {
        for peer in &self.peers {
            if *peer != self.peer_id {
                self.timer
                    .schedule(TimerEvent::Fire(Command::ParticipationObserved {
                        peer_id: *peer,
                        freshness: 0,
                        current_marker: 0,
                    }));
            }
        }

        self.timer
            .schedule(TimerEvent::Fire(Command::LocalParticipationCompleted));
    }

    pub fn step(&mut self, current_marker: Freshness, use_timer: bool) {
        if use_timer {
            if let Some(event) = self.timer.poll() {
                let TimerEvent::Fire(command) = event;
                self.protocol.evaluate(command);
                return;
            }
        }

        if let Some((_, command)) = self.transport.recv() {
            let outcomes = self.protocol.evaluate(command);

            for outcome in &outcomes {
                if let Outcome::BroadcastLocalReady = outcome {
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
            }
        }
    }
}
