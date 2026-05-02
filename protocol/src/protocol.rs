// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec;
use alloc::vec::Vec;

use faction::PeerId;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::faction::Faction;
use faction::process_result::ProcessResult;

use crate::input_message::InputMessage;
use crate::message_translator::MessageTranslator;
use crate::output_message::OutputMessage;
use crate::timer_event::TimerEvent;
use crate::timer_message::TimerMessage;

pub struct Protocol {
    faction: Faction,
    peers: Vec<PeerId>,
    local_peer_id: PeerId,
    translator: MessageTranslator,
}

impl Protocol {
    pub fn new(faction: Faction, peers: Vec<PeerId>, local_peer_id: PeerId) -> Self {
        Self {
            faction,
            peers,
            local_peer_id,
            translator: MessageTranslator::new(),
        }
    }

    pub fn cluster_view(&mut self) -> ClusterView {
        match self.faction.process(Command::Probe) {
            ProcessResult::Probed { cluster_view, .. } => cluster_view,
            _ => unreachable!(),
        }
    }

    pub fn initialize(&self) -> Vec<OutputMessage> {
        let mut actions = Vec::new();

        for peer in &self.peers {
            if *peer != self.local_peer_id {
                actions.push(OutputMessage::Schedule(TimerEvent::Fire(
                    TimerMessage::ParticipationObserved { peer_id: *peer },
                )));
            }
        }

        actions.push(OutputMessage::Schedule(TimerEvent::Fire(
            TimerMessage::LocalParticipationCompleted,
        )));

        actions.push(OutputMessage::BroadcastPing);
        actions.push(OutputMessage::Schedule(TimerEvent::Fire(
            TimerMessage::RetryPing,
        )));

        actions
    }

    pub fn decide(&mut self, input_message: InputMessage) -> Vec<OutputMessage> {
        if let InputMessage::Timer(TimerMessage::RetryPing) = &input_message {
            if self.cluster_view().is_exited() {
                vec![OutputMessage::Noop]
            } else {
                vec![
                    OutputMessage::BroadcastPing,
                    OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryPing)),
                ]
            }
        } else if let InputMessage::Timer(TimerMessage::RetryReady) = &input_message {
            if self.cluster_view().is_exited() {
                vec![OutputMessage::Noop]
            } else {
                vec![
                    OutputMessage::BroadcastReady,
                    OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady)),
                ]
            }
        } else {
            let command = self.translator.to_command(input_message);

            match self.faction.process(command) {
                ProcessResult::Accepted { outcomes, .. } => {
                    self.translator.to_output_messages(outcomes)
                }
                ProcessResult::Probed { .. } | ProcessResult::Rejected { .. } => {
                    vec![OutputMessage::Noop]
                }
            }
        }
    }
}
