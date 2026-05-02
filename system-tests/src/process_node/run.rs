// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::peer_state::PeerState;

use crate::faction_node::FactionNode;

pub fn run(mut node: FactionNode) -> PeerState {
    node.run();
    node.peer_state()
}
