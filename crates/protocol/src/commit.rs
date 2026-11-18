// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction::instance::peer_id::PeerId;
use faction::types::Epoch;

// A change the layer above the node has committed, at the epoch it numbered
// it (§16): an addition, a removal or a new required count -- the commands
// that layer alone reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Commit<P: PeerId = u64> {
    Addition { peer_id: P, epoch: Epoch },
    Removal { peer_id: P, epoch: Epoch },
    RequiredCount { required_count: usize, epoch: Epoch },
}

impl<P: PeerId> Commit<P> {
    // The command faction is given for the commit.
    #[must_use]
    pub fn command(self) -> Command<P> {
        match self {
            Self::Addition { peer_id, epoch } => Command::AdditionCommitted { peer_id, epoch },
            Self::Removal { peer_id, epoch } => Command::RemovalCommitted { peer_id, epoch },
            Self::RequiredCount {
                required_count,
                epoch,
            } => Command::RequiredCountChanged {
                required_count,
                epoch,
            },
        }
    }
}
