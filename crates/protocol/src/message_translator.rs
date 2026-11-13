// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use faction::command::Command;
use faction::outcome::Outcome;

use crate::input_message::InputMessage;
use crate::output_message::OutputMessage;
use crate::timer_event::TimerEvent;
use crate::timer_message::TimerMessage;
use crate::transport_message::TransportMessage;

pub struct MessageTranslator;

impl MessageTranslator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for MessageTranslator {
    fn default() -> Self {
        Self
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
            },
            InputMessage::Timer(msg) => match msg {
                TimerMessage::ParticipationObserved { peer_id } => {
                    Command::ParticipationObserved { peer_id }
                }
                TimerMessage::LocalParticipationCompleted => Command::LocalParticipationCompleted,
                TimerMessage::RetryPing => unreachable!("handled in decide()"),
                TimerMessage::RetryReady => unreachable!("handled in decide()"),
                TimerMessage::DeadlineExpired => Command::DeadlineExpired,
            },
        }
    }

    // Every outcome of a result, in order, each turned into what the caller
    // must do; a result asking nothing is a no-op.
    #[must_use]
    pub fn to_output_messages(&self, outcomes: Vec<Outcome>) -> Vec<OutputMessage> {
        let messages: Vec<OutputMessage> = outcomes.into_iter().flat_map(Self::messages).collect();
        if messages.is_empty() {
            vec![OutputMessage::Noop]
        } else {
            messages
        }
    }

    fn messages(outcome: Outcome) -> Vec<OutputMessage> {
        match outcome {
            Outcome::BroadcastLocalReady => vec![
                OutputMessage::BroadcastReady,
                OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady)),
            ],
            Outcome::Concluded { .. } => vec![
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::LocalParticipationCompleted)),
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryPing)),
                OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryReady)),
            ],
            Outcome::AcknowledgeRejoin { .. } => vec![OutputMessage::BroadcastReady],
            _ => Vec::new(),
        }
    }
}
