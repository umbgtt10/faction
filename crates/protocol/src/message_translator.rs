// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::instance::peer_id::PeerId;
use faction::outcome::Outcome;
use faction::types::Epoch;

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
    // The command a message is, a peer's signal at the epoch it carries; a
    // signal of the node's own timers at `epoch`, the node's.
    #[must_use]
    pub fn to_command<P: PeerId>(&self, message: InputMessage<P>, epoch: Epoch) -> Command<P> {
        match message {
            InputMessage::Transport(msg) => match msg {
                TransportMessage::Ping { from, epoch } => Command::ParticipationObserved {
                    peer_id: from,
                    epoch,
                },
                TransportMessage::Ready { from, epoch } => Command::ReadyObserved {
                    peer_id: from,
                    epoch,
                },
                TransportMessage::Bootstrapped { .. } => Command::Probe,
                TransportMessage::MembershipAcknowledged { from, epoch } => {
                    Command::MembershipAcknowledged {
                        peer_id: from,
                        epoch,
                    }
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
                    Command::ParticipationObserved { peer_id, epoch }
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
    // must do -- the membership a peer is sent, and the epoch a broadcast
    // carries, read from the result's view, as the node `local_peer_id` sends
    // them; a result asking nothing is a no-op.
    #[must_use]
    pub fn to_output_messages<P: PeerId>(
        &self,
        outcomes: Vec<Outcome<P>>,
        view: &ClusterView<P>,
        local_peer_id: P,
    ) -> Vec<OutputMessage<P>> {
        let messages: Vec<OutputMessage<P>> = outcomes
            .into_iter()
            .flat_map(|outcome| self.messages(outcome, view, local_peer_id))
            .collect();
        if messages.is_empty() {
            vec![OutputMessage::Noop]
        } else {
            messages
        }
    }

    fn messages<P: PeerId>(
        &self,
        outcome: Outcome<P>,
        view: &ClusterView<P>,
        local_peer_id: P,
    ) -> Vec<OutputMessage<P>> {
        match outcome {
            Outcome::BroadcastLocalReady => vec![
                OutputMessage::BroadcastReady {
                    epoch: view.epoch(),
                },
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
            Outcome::AliveNoted { peer_id }
            | Outcome::PeerRevived { peer_id }
            | Outcome::MemberRemoved { peer_id } => vec![
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::ProbeMissed { peer_id })),
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::IndirectProbesMissed {
                    peer_id,
                })),
            ],
            Outcome::AcknowledgeRejoin { peer_id } => vec![OutputMessage::Send {
                to: peer_id,
                message: TransportMessage::Ready {
                    from: local_peer_id,
                    epoch: view.epoch(),
                },
            }],
            Outcome::SendMembership { peer_id } => vec![
                OutputMessage::Send {
                    to: peer_id,
                    message: TransportMessage::Membership {
                        from: local_peer_id,
                        to: peer_id,
                        members: view.members().as_slice().to_vec(),
                        required_count: view.required_count(),
                        epoch: view.epoch(),
                    },
                },
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryMembership { peer_id })),
                OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryMembership {
                    peer_id,
                })),
            ],
            Outcome::ProposeAddition { peer_id } => vec![
                OutputMessage::ProposeAddition {
                    peer_id,
                    epoch: view.epoch(),
                },
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryMembership { peer_id })),
            ],
            Outcome::ProposeRemoval { peer_id } => vec![OutputMessage::ProposeRemoval {
                peer_id,
                epoch: view.epoch(),
            }],
            Outcome::AdditionWithdrawn { peer_id } => vec![OutputMessage::Cancel(
                TimerEvent::Fire(TimerMessage::RetryMembership { peer_id }),
            )],
            _ => Vec::new(),
        }
    }
}
