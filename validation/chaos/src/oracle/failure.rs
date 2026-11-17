// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use faction::types::PeerId;
use faction_system_tests::infra::invariants::breach::Breach;
use faction_system_tests::node_snapshot::NodeSnapshot;

// How a scenario failed: a node broke an invariant while it ran; the end
// state was not reached by the deadline -- the first node short of it, what
// it was expected to show and what it showed; or a planned change never
// committed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    Breach {
        round: u64,
        breach: Breach,
    },
    Diverged {
        round: u64,
        peer_id: PeerId,
        expected: String,
        found: Box<NodeSnapshot>,
    },
    Uncommitted {
        round: u64,
    },
}

impl Display for Failure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Breach { round, breach } => write!(
                formatter,
                "round {round}: node {} broke \"{}\" on {:?}",
                breach.peer_id, breach.invariant, breach.command
            ),
            Self::Diverged {
                round,
                peer_id,
                expected,
                found,
            } => write!(
                formatter,
                "round {round}: node {peer_id} expected {expected}, found {found:?}"
            ),
            Self::Uncommitted { round } => {
                write!(formatter, "round {round}: a planned change never committed")
            }
        }
    }
}

impl Error for Failure {}
