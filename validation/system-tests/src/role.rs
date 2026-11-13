// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::newcomer::Newcomer;
use faction_protocol::protocol::Protocol;

// What a node runs: a newcomer waiting for its membership, or the protocol of
// a node that has one.
pub enum Role {
    Waiting(Newcomer),
    Running(Box<Protocol>),
}
