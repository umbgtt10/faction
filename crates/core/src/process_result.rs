// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use crate::instance::peer_id::PeerId;
use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command_kind::CommandKind;
use crate::outcome::Outcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessResult<P: PeerId = u64> {
    Accepted {
        cluster_view: ClusterView<P>,
        admissible: Vec<CommandKind>,
        outcomes: Vec<Outcome<P>>,
    },
    Rejected {
        cluster_view: ClusterView<P>,
        admissible: Vec<CommandKind>,
    },
    Probed {
        cluster_view: ClusterView<P>,
        admissible: Vec<CommandKind>,
    },
}
