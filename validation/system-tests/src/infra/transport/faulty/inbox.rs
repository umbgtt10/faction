// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::VecDeque;

use faction_protocol::transport_message::TransportMessage;

use crate::infra::dice::Dice;
use crate::infra::transport::faulty::fault_policy::FaultPolicy;
use crate::infra::transport::wire_codec::WireCodec;

const FRAGMENT_BYTES: usize = 32;

// A node's receive ring, drained by a consumer that may be busy: past its
// capacity it drops the tail, and at a backlog it loses a message by its size
// -- a fragment of it lost at each roll -- and by the backlog itself, a burst
// losing more the longer it is. An idle ring loses nothing. Its draws are its
// own, from a seed.
pub struct Inbox {
    messages: VecDeque<TransportMessage>,
    dice: Dice,
}

impl Inbox {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            messages: VecDeque::new(),
            dice: Dice::new(seed),
        }
    }

    // A message arrived: kept, or returned with the fault that lost it.
    pub fn arrive(
        &mut self,
        message: TransportMessage,
        policy: &FaultPolicy,
    ) -> Option<(&'static str, TransportMessage)> {
        let backlog = self.messages.len();
        if policy.inbox > 0 && backlog >= policy.inbox {
            return Some(("INBOX FULL (tail dropped)", message));
        }
        if backlog > 0 {
            if self.loses_a_fragment(&message, policy.size_loss) {
                return Some(("SIZE (a fragment lost at a backlog)", message));
            }
            if self.dice.rolls(Self::burst(policy.burst_loss, backlog)) {
                return Some(("BURST (lost to a backlog)", message));
            }
        }
        self.messages.push_back(message);
        None
    }

    pub fn take(&mut self) -> Option<TransportMessage> {
        self.messages.pop_front()
    }

    fn burst(percent: u8, backlog: usize) -> u8 {
        let scaled = usize::from(percent).saturating_mul(backlog).min(100);
        u8::try_from(scaled).unwrap_or(100)
    }

    fn loses_a_fragment(&mut self, message: &TransportMessage, percent: u8) -> bool {
        let fragments = WireCodec::encode(message).len().div_ceil(FRAGMENT_BYTES);
        (0..fragments).any(|_| self.dice.rolls(percent))
    }
}
