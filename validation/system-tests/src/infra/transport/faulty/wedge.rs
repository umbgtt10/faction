// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

use crate::infra::transport::faulty::fault_policy::FaultPolicy;

// Head-of-line blocking, as a transmit ring holding a datagram no neighbour
// answers for: under it, the first send to an unreachable peer wedges every
// send after it, to anyone, until that peer is reachable again.
#[derive(Debug, Default)]
pub struct Wedge {
    on: Option<PeerId>,
}

impl Wedge {
    // Whether a send to `to` is wedged behind an undeliverable one.
    pub fn blocks(&mut self, to: PeerId, policy: &FaultPolicy) -> bool {
        if !policy.head_of_line {
            self.on = None;
            return false;
        }
        self.on = self.on.filter(|peer| policy.unreachable.contains(peer));
        if self.on.is_none() && policy.unreachable.contains(&to) {
            self.on = Some(to);
        }
        self.on.is_some()
    }
}
