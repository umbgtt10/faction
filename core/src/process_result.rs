// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

extern crate alloc;

use alloc::vec::Vec;

use crate::command::Command;
use crate::outcome::Outcome;
use crate::snapshot::Snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessResult {
    Accepted {
        outcomes: Vec<Outcome>,
        snapshot: Snapshot,
    },
    Rejected {
        snapshot: Snapshot,
        admissible: Vec<Command>,
    },
    Snapshot {
        snapshot: Snapshot,
    },
}
