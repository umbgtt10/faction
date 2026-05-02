// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
    pub fn to_output_messages(&self, outcomes: Vec<Outcome>) -> Vec<OutputMessage> {
        for outcome in outcomes {
            match outcome {
                Outcome::BroadcastLocalReady => {
                    return vec![
                        OutputMessage::BroadcastReady,
                        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady)),
                    ];
                }
                Outcome::Concluded { .. } => {
                    return vec![
                        OutputMessage::Cancel(TimerEvent::Fire(
                            TimerMessage::LocalParticipationCompleted,
                        )),
                        OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryPing)),
                        OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryReady)),
                    ];
                }
                _ => {}
            }
        }

        vec![OutputMessage::Noop]
    }

    #[must_use]
    pub fn to_command(&self, message: InputMessage) -> Command {
        match message {
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
                TimerMessage::RetryPing => unreachable!("handled in decide()"),
                TimerMessage::RetryReady => unreachable!("handled in decide()"),
                TimerMessage::DeadlineExpired => Command::DeadlineExpired,
            },
        }
    }
}
