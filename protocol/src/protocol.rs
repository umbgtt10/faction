// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec;
use alloc::vec::Vec;

use faction::PeerId;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::faction::Faction;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;

use crate::input_message::InputMessage;
use crate::output_message::OutputMessage;
use crate::timer_event::TimerEvent;
use crate::timer_message::TimerMessage;
use crate::transport_message::TransportMessage;

pub struct Protocol {
    faction: Faction,
    peers: Vec<PeerId>,
    local_peer_id: PeerId,
}

impl Protocol {
    pub fn cluster_view(&mut self) -> ClusterView {
        match self.faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        }
    }

    pub fn new(faction: Faction, peers: Vec<PeerId>, local_peer_id: PeerId) -> Self {
        Self {
            faction,
            peers,
            local_peer_id,
        }
    }

    pub fn start_decisions(&self) -> Vec<OutputMessage> {
        let mut decisions = Vec::new();

        for peer in &self.peers {
            if *peer != self.local_peer_id {
                decisions.push(OutputMessage::Schedule(TimerEvent::Fire(
                    TimerMessage::ParticipationObserved { peer_id: *peer },
                )));
            }
        }

        decisions.push(OutputMessage::Schedule(TimerEvent::Fire(
            TimerMessage::LocalParticipationCompleted,
        )));

        decisions
    }

    pub fn decide(&mut self, message: InputMessage) -> Vec<OutputMessage> {
        let command = match message {
            InputMessage::Transport(msg) => match msg {
                TransportMessage::Ping { from } => Command::ParticipationObserved {
                    peer_id: from,
                    freshness: 0,
                    current_marker: 0,
                },
                TransportMessage::Ready { from } => Command::ReadyObserved {
                    peer_id: from,
                    freshness: 0,
                    current_marker: 0,
                },
                TransportMessage::Bootstrapped { .. } => Command::Probe,
            },
            InputMessage::Timer(msg) => match msg {
                TimerMessage::ParticipationObserved { peer_id } => Command::ParticipationObserved {
                    peer_id,
                    freshness: 0,
                    current_marker: 0,
                },
                TimerMessage::LocalParticipationCompleted => Command::LocalParticipationCompleted,
                TimerMessage::DeadlineExpired => Command::DeadlineExpired,
            },
        };

        let outcomes = match self.faction.process(command) {
            ProcessResult::Accepted { outcomes, .. } => outcomes,
            ProcessResult::Probed { .. } => unreachable!(),
            ProcessResult::Rejected { .. } => return vec![OutputMessage::Noop],
        };

        for outcome in outcomes {
            match outcome {
                Outcome::BroadcastLocalReady => return vec![OutputMessage::BroadcastReady],
                Outcome::Exited { .. } => {
                    return vec![OutputMessage::Cancel(TimerEvent::Fire(
                        TimerMessage::LocalParticipationCompleted,
                    ))];
                }
                _ => {}
            }
        }

        vec![OutputMessage::Noop]
    }
}
