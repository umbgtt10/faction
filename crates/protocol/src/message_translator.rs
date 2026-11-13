// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::outcome::Outcome;
use faction::types::PeerId;

use crate::input_message::InputMessage;
use crate::output_message::OutputMessage;
use crate::timer_event::TimerEvent;
use crate::timer_message::TimerMessage;
use crate::transport_message::TransportMessage;

// Messages in, commands out; outcomes in, what the caller must do out. A
// missed probe is followed up through at most `helpers` of the members it
// lists.
pub struct MessageTranslator {
    helpers: usize,
}

impl MessageTranslator {
    #[must_use]
    pub fn new(helpers: usize) -> Self {
        Self { helpers }
    }
}

// By default, every member a missed probe lists is asked.
impl Default for MessageTranslator {
    fn default() -> Self {
        Self::new(usize::MAX)
    }
}

impl MessageTranslator {
    #[must_use]
    pub fn to_command(&self, message: InputMessage) -> Command {
        match message {
            InputMessage::Transport(msg) => match msg {
                TransportMessage::Ping { from } => Command::ParticipationObserved { peer_id: from },
                TransportMessage::Ready { from } => Command::ReadyObserved { peer_id: from },
                TransportMessage::Bootstrapped { .. } => Command::Probe,
                TransportMessage::MembershipAcknowledged { from } => {
                    Command::MembershipAcknowledged { peer_id: from }
                }
                TransportMessage::Membership { .. } => unreachable!("handled in decide()"),
                TransportMessage::ProbeAnswer {
                    from,
                    relay_for: None,
                } => Command::PeerHeard { peer_id: from },
                TransportMessage::IndirectProbeAnswer { target, .. } => {
                    Command::PeerHeard { peer_id: target }
                }
                TransportMessage::Probe { .. }
                | TransportMessage::IndirectProbe { .. }
                | TransportMessage::ProbeAnswer { .. } => unreachable!("handled in decide()"),
            },
            InputMessage::Timer(msg) => match msg {
                TimerMessage::ParticipationObserved { peer_id } => {
                    Command::ParticipationObserved { peer_id }
                }
                TimerMessage::LocalParticipationCompleted => Command::LocalParticipationCompleted,
                TimerMessage::RetryPing => unreachable!("handled in decide()"),
                TimerMessage::RetryReady => unreachable!("handled in decide()"),
                TimerMessage::DeadlineExpired => Command::DeadlineExpired,
                TimerMessage::RetryMembership { .. } => unreachable!("handled in decide()"),
                TimerMessage::ProbeDue => unreachable!("handled in decide()"),
                TimerMessage::ProbeMissed { peer_id } => Command::ProbeMissed { peer_id },
                TimerMessage::IndirectProbesMissed { peer_id } => {
                    Command::IndirectProbesMissed { peer_id }
                }
            },
        }
    }

    // Every outcome of a result, in order, each turned into what the caller
    // must do -- the membership a newcomer is sent read from the result's view,
    // as the node `local_peer_id` sends it; a result asking nothing is a no-op.
    #[must_use]
    pub fn to_output_messages(
        &self,
        outcomes: Vec<Outcome>,
        view: &ClusterView,
        local_peer_id: PeerId,
    ) -> Vec<OutputMessage> {
        let messages: Vec<OutputMessage> = outcomes
            .into_iter()
            .flat_map(|outcome| self.messages(outcome, view, local_peer_id))
            .collect();
        if messages.is_empty() {
            vec![OutputMessage::Noop]
        } else {
            messages
        }
    }

    fn messages(
        &self,
        outcome: Outcome,
        view: &ClusterView,
        local_peer_id: PeerId,
    ) -> Vec<OutputMessage> {
        match outcome {
            Outcome::BroadcastLocalReady => vec![
                OutputMessage::BroadcastReady,
                OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady)),
            ],
            Outcome::Concluded { .. } => vec![
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::LocalParticipationCompleted)),
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryPing)),
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryReady)),
                OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ProbeDue)),
            ],
            Outcome::SendProbe { peer_id } => vec![
                OutputMessage::Send {
                    to: peer_id,
                    message: TransportMessage::Probe {
                        from: local_peer_id,
                        relay_for: None,
                    },
                },
                OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::ProbeMissed { peer_id })),
            ],
            Outcome::SendIndirectProbes { peer_id, via } => via
                .into_iter()
                .take(self.helpers)
                .map(|helper| OutputMessage::Send {
                    to: helper,
                    message: TransportMessage::IndirectProbe {
                        from: local_peer_id,
                        target: peer_id,
                    },
                })
                .chain([OutputMessage::Schedule(TimerEvent::Fire(
                    TimerMessage::IndirectProbesMissed { peer_id },
                ))])
                .collect(),
            Outcome::AliveNoted { peer_id } | Outcome::PeerRevived { peer_id } => vec![
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::ProbeMissed { peer_id })),
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::IndirectProbesMissed {
                    peer_id,
                })),
            ],
            Outcome::AcknowledgeRejoin { .. } => vec![OutputMessage::BroadcastReady],
            Outcome::SendMembership { peer_id } => vec![
                OutputMessage::Send {
                    to: peer_id,
                    message: TransportMessage::Membership {
                        from: local_peer_id,
                        members: view.members().as_slice().to_vec(),
                        required_count: view.required_count(),
                    },
                },
                OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryMembership {
                    peer_id,
                })),
            ],
            Outcome::ProposeAddition { peer_id } => vec![
                OutputMessage::ProposeAddition { peer_id },
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryMembership { peer_id })),
            ],
            Outcome::AdditionWithdrawn { peer_id } => vec![OutputMessage::Cancel(
                TimerEvent::Fire(TimerMessage::RetryMembership { peer_id }),
            )],
            _ => Vec::new(),
        }
    }
}
