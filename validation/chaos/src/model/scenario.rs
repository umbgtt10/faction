// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::model::fault_window::FaultWindow;
use crate::model::planned_change::PlannedChange;
use crate::model::restart_plan::RestartPlan;

// A scenario, drawn from its seed: the cluster -- its size, its required
// count, its suspicion threshold -- the changes the layer above carries
// through, and the chaos it plays them under: faults, restarts, commits
// reaching a node up to `delivery_lag` rounds late and, at `duplicates`
// percent, twice. The chaos lasts `chaos_rounds`; quiescence then has
// `quiescence_rounds` to reach the end state the committed log implies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub seed: u64,
    pub nodes: usize,
    pub required_count: usize,
    pub suspicion_threshold: usize,
    pub changes: Vec<PlannedChange>,
    pub faults: Vec<FaultWindow>,
    pub restarts: Vec<RestartPlan>,
    pub delivery_lag: u64,
    pub duplicates: u8,
    pub chaos_rounds: u64,
    pub quiescence_rounds: u64,
}
