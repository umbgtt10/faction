// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::types::PeerId;

// A node and an incarnation: what tells one instance's receipts from
// another's. The caller guarantees an incarnation never repeats for its node,
// so a node created again -- restarted, or a removed node invited back -- is a
// new instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstanceId {
    pub node: PeerId,
    pub incarnation: u64,
}
