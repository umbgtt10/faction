// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use faction::instance::peer_id::PeerId;
use faction::types::Epoch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportMessage<P: PeerId = u64> {
    Ping {
        from: P,
        epoch: Epoch,
    },
    Ready {
        from: P,
        epoch: Epoch,
    },
    Bootstrapped {
        from: P,
    },
    Membership {
        from: P,
        to: P,
        members: Vec<P>,
        required_count: usize,
        epoch: Epoch,
    },
    MembershipAcknowledged {
        from: P,
        epoch: Epoch,
    },
    Probe {
        from: P,
        relay_for: Option<P>,
    },
    ProbeAnswer {
        from: P,
        relay_for: Option<P>,
    },
    IndirectProbe {
        from: P,
        target: P,
    },
    IndirectProbeAnswer {
        from: P,
        target: P,
    },
}
