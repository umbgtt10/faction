// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::transport::faulty::message_kind::MessageKind;

/// One `0..=100` percentage per transport fault, plus the seed that makes every
/// misbehaving decision reproducible.
#[derive(Debug, Clone, Copy)]
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

impl FaultPolicy {
    #[must_use]
    pub fn none() -> FaultPolicy {
        FaultPolicy {
            loss: 0,
            duplication: 0,
            delay: 0,
            reorder: 0,
            partition: 0,
            asymmetric: 0,
            selective: 0,
            selective_target: MessageKind::Ready,
            seed: 0,
        }
    }
}

impl Default for FaultPolicy {
    fn default() -> FaultPolicy {
        FaultPolicy::none()
    }
}
