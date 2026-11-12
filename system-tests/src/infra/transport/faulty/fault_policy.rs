// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::infra::transport::faulty::message_kind::MessageKind;

/// One `0..=100` percentage per transport fault, plus the seed that makes every
/// misbehaving decision reproducible.
#[derive(Debug, Clone, Copy, Default)]
pub struct FaultPolicy {
    pub loss: u8,
    pub duplication: u8,
    pub delay: u8,
    pub reorder: u8,
    pub partition: u8,
    pub asymmetric: u8,
    pub selective: u8,
    pub selective_target: MessageKind,
    pub seed: u64,
}
