// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::outcome::Outcome;
use faction::process_result::ProcessResult;

// What a node answered, read the way a test asserts it.
pub struct Answers;

impl Answers {
    pub fn accepted(result: ProcessResult) -> (Vec<Outcome>, ClusterView, Vec<Command>) {
        match result {
            ProcessResult::Accepted {
                outcomes,
                cluster_view,
                admissible,
            } => (outcomes, cluster_view, admissible),
            other => panic!("expected Accepted, got {other:?}"),
        }
    }

    pub fn is_rejected(result: &ProcessResult) -> bool {
        matches!(result, ProcessResult::Rejected { .. })
    }
}
