// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use faction::types::Epoch;
use faction::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportMessage {
    Ping {
        from: PeerId,
        epoch: Epoch,
    },
    Ready {
        from: PeerId,
        epoch: Epoch,
    },
    Bootstrapped {
        from: PeerId,
    },
    Membership {
        from: PeerId,
        members: Vec<PeerId>,
        required_count: usize,
        epoch: Epoch,
    },
    MembershipAcknowledged {
        from: PeerId,
        epoch: Epoch,
    },
    Probe {
        from: PeerId,
        relay_for: Option<PeerId>,
    },
    ProbeAnswer {
        from: PeerId,
        relay_for: Option<PeerId>,
    },
    IndirectProbe {
        from: PeerId,
        target: PeerId,
    },
    IndirectProbeAnswer {
        from: PeerId,
        target: PeerId,
    },
}
