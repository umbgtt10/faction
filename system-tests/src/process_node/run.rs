// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::peer_state::PeerState;

use crate::faction_node::FactionNode;

pub fn run(mut node: FactionNode) -> PeerState {
    node.run();
    node.peer_state()
}
