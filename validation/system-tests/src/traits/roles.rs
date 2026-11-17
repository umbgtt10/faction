// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::instance::instance_id::InstanceId;

use crate::membership::Membership;
use crate::role::Role;

// What makes a node's role anew, as the instance it is given: a running node
// created at a membership -- a restart -- and a newcomer waiting for its
// membership -- a removed node invited back, which joins as any newcomer does.
pub trait Roles: Send {
    fn running(&self, instance: InstanceId, membership: &Membership) -> Role;

    fn waiting(&self, instance: InstanceId) -> Role;
}
