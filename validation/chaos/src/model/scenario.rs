// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use serde::Serialize;

use crate::model::fault_window::FaultWindow;
use crate::model::planned_change::PlannedChange;
use crate::model::restart_plan::RestartPlan;

// A scenario, drawn from its seed: the cluster -- its size, its required
// count, its suspicion threshold, each peer's bound of changes in flight,
// by its id, 1 past the list -- the changes the layer above carries
// through, and the chaos it plays them under: faults, restarts, commits
// reaching a node up to `delivery_lag` rounds late and, at `duplicates`
// percent, twice. The chaos lasts `chaos_rounds`; quiescence then has
// `quiescence_rounds` to reach the end state the committed log implies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scenario {
    pub seed: u64,
    pub nodes: usize,
    pub required_count: usize,
    pub suspicion_threshold: usize,
    #[serde(default)]
    pub changes_in_flight: Vec<usize>,
    pub changes: Vec<PlannedChange>,
    pub faults: Vec<FaultWindow>,
    pub restarts: Vec<RestartPlan>,
    pub delivery_lag: u64,
    pub duplicates: u8,
    pub chaos_rounds: u64,
    pub quiescence_rounds: u64,
}
