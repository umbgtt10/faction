// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::model::change::Change;
use crate::model::scenario::Scenario;

// The peers a scenario's layer above may ask to coordinate a change: every
// peer it runs that no change removes, in the order of their ids -- its
// genesis members, and the newcomers its changes add, which refuse while
// their membership is provisional and coordinate once their own addition
// commits.
pub struct Coordinators;

impl Coordinators {
    #[must_use]
    pub fn of(scenario: &Scenario) -> Vec<u64> {
        let removed: Vec<u64> = scenario
            .changes
            .iter()
            .filter_map(|planned| match planned.change {
                Change::Removal { peer_id } => Some(peer_id),
                _ => None,
            })
            .collect();
        let added = scenario
            .changes
            .iter()
            .filter_map(|planned| match planned.change {
                Change::Addition { peer_id } => Some(peer_id),
                _ => None,
            });
        let mut coordinators: Vec<u64> = (0..scenario.nodes as u64)
            .chain(added)
            .filter(|peer_id| !removed.contains(peer_id))
            .collect();
        coordinators.sort_unstable();
        coordinators.dedup();
        coordinators
    }
}
