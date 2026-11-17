// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Serialize;

use faction::types::PeerId;

use crate::infra::transport::faulty::message_kind::MessageKind;

/// A node's transport faults, plus the seed that makes every misbehaving decision
/// reproducible; the default is a faithful transport.
///
/// On its sends: a `0..=100` percentage each for loss, duplication, delay,
/// reordering, cut links, one-way links, the loss of one kind of message and a
/// message delivered to another peer than its addressee; how many of the first
/// messages to each peer are lost; the peers no send reaches, and whether a send to
/// one wedges every send after it. On its receives: the capacity of its receive
/// ring, none at 0, and the percentages a busy ring loses a message at, per
/// fragment of it and per message waiting. And a node mute, deaf, or starved --
/// its sends held and nothing received until it is fed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultPolicy {
    pub loss: u8,
    pub duplication: u8,
    pub delay: u8,
    pub reorder: u8,
    pub partition: u8,
    pub asymmetric: u8,
    pub selective: u8,
    pub selective_target: MessageKind,
    pub misdelivery: u8,
    pub first_contact_lost: u8,
    pub unreachable: Vec<PeerId>,
    pub head_of_line: bool,
    pub inbox: usize,
    pub size_loss: u8,
    pub burst_loss: u8,
    pub mute: bool,
    pub deaf: bool,
    pub starved: bool,
    pub seed: u64,
}
