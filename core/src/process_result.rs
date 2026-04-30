// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec::Vec;

use crate::command::Command;
use crate::outcome::Outcome;
use crate::cluster_view::ClusterView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessResult {
    Accepted {
        outcomes: Vec<Outcome>,
        cluster_view: ClusterView,
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
