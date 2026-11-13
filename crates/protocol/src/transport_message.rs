// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use faction::types::PeerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportMessage {
    Ping {
        from: PeerId,
    },
    Ready {
        from: PeerId,
    },
    Bootstrapped {
        from: PeerId,
    },
    Membership {
        from: PeerId,
        members: Vec<PeerId>,
        required_count: usize,
    },
    MembershipAcknowledged {
        from: PeerId,
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
