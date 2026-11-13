// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

extern crate alloc;

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::outcome::Outcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessResult {
    Accepted {
        cluster_view: ClusterView,
        admissible: Vec<Command>,
        outcomes: Vec<Outcome>,
    },
    Rejected {
        cluster_view: ClusterView,
        admissible: Vec<Command>,
    },
    Probed {
        cluster_view: ClusterView,
        admissible: Vec<Command>,
    },
}
