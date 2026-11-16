// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

use crate::membership::Membership;
use crate::role::Role;

// What makes a node's role anew: a running node created at a membership -- a
// restart -- and a newcomer waiting for its membership -- a removed node
// invited back, which joins as any newcomer does.
pub trait Roles: Send {
    fn running(&self, peer_id: PeerId, membership: &Membership) -> Role;

    fn waiting(&self, peer_id: PeerId) -> Role;
}
