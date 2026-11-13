// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use faction::addition::Addition;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::faction::Faction;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::types::PeerId;

use crate::input_message::InputMessage;
use crate::message_translator::MessageTranslator;
use crate::output_message::OutputMessage;
use crate::result_view::ResultView;
use crate::timer_event::TimerEvent;
use crate::timer_message::TimerMessage;
use crate::transport_message::TransportMessage;

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

    pub fn abort_addition(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::AdditionAborted { peer_id })
    }

    pub fn admit(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::JoinApproved { peer_id })
    }

    pub fn change_required_count(&mut self, required_count: usize) -> Vec<OutputMessage> {
        self.apply(Command::RequiredCountChanged { required_count })
    }

    pub fn cluster_view(&mut self) -> ClusterView {
        ResultView::of(self.faction.process(Command::Probe))
    }

    pub fn commit_addition(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::AdditionCommitted { peer_id })
    }

    pub fn decide(&mut self, input_message: InputMessage) -> Vec<OutputMessage> {
        match input_message {
            InputMessage::Timer(TimerMessage::RetryPing) => {
                self.retry(OutputMessage::BroadcastPing, TimerMessage::RetryPing)
            }
            InputMessage::Timer(TimerMessage::RetryReady) => {
                self.retry(OutputMessage::BroadcastReady, TimerMessage::RetryReady)
            }
            InputMessage::Timer(TimerMessage::RetryMembership { peer_id }) => {
                self.resend_membership(peer_id)
            }
            InputMessage::Transport(TransportMessage::Membership { from, .. }) => {
                vec![OutputMessage::Send {
                    to: from,
                    message: TransportMessage::MembershipAcknowledged {
                        from: self.local_peer_id,
                    },
                }]
            }
            other => {
                let command = self.translator.to_command(other);
                self.apply(command)
            }
        }
    }

    pub fn deny(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::JoinRejected { peer_id })
    }

    pub fn expire_deadline(&mut self) -> Vec<OutputMessage> {
        self.apply(Command::DeadlineExpired)
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

    pub fn request_addition(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::AdditionRequested { peer_id })
    }

    pub fn request_join(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::JoinRequested { peer_id })
    }

    fn apply(&mut self, command: Command) -> Vec<OutputMessage> {
        match self.faction.process(command) {
            ProcessResult::Accepted {
                outcomes,
                cluster_view,
                ..
            } => self
                .translator
                .to_output_messages(outcomes, &cluster_view, self.local_peer_id),
            ProcessResult::Probed { .. } | ProcessResult::Rejected { .. } => {
                vec![OutputMessage::Noop]
            }
        }
    }

    // The membership sent again while its addition waits for the newcomer's
    // acknowledgement, and its retry with it; nothing once it has come.
    fn resend_membership(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        let view = self.cluster_view();
        if view.addition() == (Addition::Requested { peer_id }) {
            self.translator.to_output_messages(
                vec![Outcome::SendMembership { peer_id }],
                &view,
                self.local_peer_id,
            )
        } else {
            vec![OutputMessage::Noop]
        }
    }

    // A broadcast repeated, and its retry with it, until the node concludes.
    fn retry(&mut self, broadcast: OutputMessage, retry: TimerMessage) -> Vec<OutputMessage> {
        if self.cluster_view().peer_state() == PeerState::Bootstrapped {
            vec![OutputMessage::Noop]
        } else {
            vec![broadcast, OutputMessage::Schedule(TimerEvent::Fire(retry))]
        }
    }
}
