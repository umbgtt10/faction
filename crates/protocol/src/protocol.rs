// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::faction::Faction;
use faction::outcome::Outcome;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::reconfiguration::Reconfiguration;
use faction::types::Epoch;
use faction::types::PeerId;

use crate::commit::Commit;
use crate::input_message::InputMessage;
use crate::message_translator::MessageTranslator;
use crate::offer::Offer;
use crate::output_message::OutputMessage;
use crate::probe_relay::ProbeRelay;
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
    // `helpers`: how many of the members a missed probe lists the node asks to
    // probe it.
    pub fn new(
        faction: Faction,
        peers: Vec<PeerId>,
        local_peer_id: PeerId,
        helpers: usize,
    ) -> Self {
        Self {
            faction,
            peers,
            local_peer_id,
            translator: MessageTranslator::new(helpers),
        }
    }

    pub fn abort_addition(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::AdditionAborted { peer_id })
    }

    pub fn abort_removal(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::RemovalAborted { peer_id })
    }

    pub fn admit(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::JoinApproved { peer_id })
    }

    pub fn cluster_view(&mut self) -> ClusterView {
        ResultView::of(self.faction.process(Command::Probe).output)
    }

    // A change the layer above has committed, at its epoch.
    pub fn commit(&mut self, commit: Commit) -> Vec<OutputMessage> {
        self.apply(commit.command())
    }

    pub fn decide(&mut self, input_message: InputMessage) -> Vec<OutputMessage> {
        match input_message {
            InputMessage::Timer(TimerMessage::RetryPing) => self.retry(TimerMessage::RetryPing),
            InputMessage::Timer(TimerMessage::RetryReady) => self.retry(TimerMessage::RetryReady),
            InputMessage::Timer(TimerMessage::RetryMembership { peer_id }) => {
                self.resend_membership(peer_id)
            }
            InputMessage::Timer(TimerMessage::ProbeDue) => self.probe_period(),
            InputMessage::Transport(
                message @ (TransportMessage::Probe { .. }
                | TransportMessage::IndirectProbe { .. }
                | TransportMessage::ProbeAnswer {
                    relay_for: Some(_), ..
                }),
            ) => ProbeRelay::reply(self.local_peer_id, &message),
            InputMessage::Transport(message @ TransportMessage::Membership { .. }) => {
                match Offer::of(&message, self.local_peer_id) {
                    Some(offer) => {
                        let taken = self.apply(offer.command());
                        offer.answered(taken, self.local_peer_id, &self.cluster_view())
                    }
                    None => vec![OutputMessage::Noop],
                }
            }
            other => {
                let epoch = self.cluster_view().epoch();
                let command = self.translator.to_command(other, epoch);
                self.apply(command)
            }
        }
    }

    // A proposal the layer above declined at `epoch`, decided again (§17).
    pub fn decline(&mut self, peer_id: PeerId, epoch: Epoch) -> Vec<OutputMessage> {
        self.apply(Command::ProposalDeclined { peer_id, epoch })
    }

    pub fn deny(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::JoinRejected { peer_id })
    }

    pub fn expire_deadline(&mut self) -> Vec<OutputMessage> {
        self.apply(Command::DeadlineExpired)
    }

    pub fn initialize(&mut self) -> Vec<OutputMessage> {
        let epoch = self.cluster_view().epoch();
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

        actions.push(OutputMessage::BroadcastPing { epoch });
        actions.push(OutputMessage::Schedule(TimerEvent::Fire(
            TimerMessage::RetryPing,
        )));

        actions
    }

    // Whether `message` invites this node, removed from its cluster, to join
    // it again: a membership addressed to it, which a removed node takes as a
    // newcomer does, created again from it.
    pub fn invites(&mut self, message: &TransportMessage) -> bool {
        Offer::of(message, self.local_peer_id).is_some()
            && self.cluster_view().peer_state() == PeerState::Removed
    }

    pub fn request_addition(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::AdditionRequested { peer_id })
    }

    pub fn request_join(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::JoinRequested { peer_id })
    }

    pub fn request_removal(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        self.apply(Command::RemovalRequested { peer_id })
    }

    fn apply(&mut self, command: Command) -> Vec<OutputMessage> {
        match self.faction.process(command).output {
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

    // A probe period on a concluded node: the member to probe probed, and the
    // next period scheduled; nothing before the node concludes.
    fn probe_period(&mut self) -> Vec<OutputMessage> {
        if self.cluster_view().peer_state() == PeerState::Bootstrapped {
            self.apply(Command::ProbeDue)
                .into_iter()
                .filter(|output| *output != OutputMessage::Noop)
                .chain([
                    OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::ProbeDue)),
                    OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ProbeDue)),
                ])
                .collect()
        } else {
            vec![OutputMessage::Noop]
        }
    }

    // The membership sent again while its addition waits for the newcomer's
    // acknowledgement, and its retry with it; nothing once it has come.
    fn resend_membership(&mut self, peer_id: PeerId) -> Vec<OutputMessage> {
        let view = self.cluster_view();
        if view
            .reconfigurations()
            .contains(&Reconfiguration::AdditionRequested { peer_id })
        {
            self.translator.to_output_messages(
                vec![Outcome::SendMembership { peer_id }],
                &view,
                self.local_peer_id,
            )
        } else {
            vec![OutputMessage::Noop]
        }
    }

    // A broadcast repeated at the node's epoch, and its retry with it, until
    // the node concludes or is removed.
    fn retry(&mut self, retry: TimerMessage) -> Vec<OutputMessage> {
        let view = self.cluster_view();
        let epoch = view.epoch();
        match (view.peer_state(), &retry) {
            (PeerState::Bootstrapped | PeerState::Removed, _) => vec![OutputMessage::Noop],
            (_, TimerMessage::RetryPing) => vec![
                OutputMessage::BroadcastPing { epoch },
                OutputMessage::Cancel(TimerEvent::Fire(retry.clone())),
                OutputMessage::Schedule(TimerEvent::Fire(retry)),
            ],
            _ => vec![
                OutputMessage::BroadcastReady { epoch },
                OutputMessage::Cancel(TimerEvent::Fire(retry.clone())),
                OutputMessage::Schedule(TimerEvent::Fire(retry)),
            ],
        }
    }
}
