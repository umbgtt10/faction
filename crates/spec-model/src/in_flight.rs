// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

use crate::node_state::NodeState;
use crate::reconfiguration::Reconfiguration;

// A node's changes in flight, in the order requested (§2, §4): one requested
// last, one acknowledged, one ended -- no two name one peer.
pub struct InFlight;

impl InFlight {
    // The changes with the addition of `peer_id`, requested, acknowledged.
    #[must_use]
    pub fn acknowledged(state: &NodeState, peer_id: &str) -> Vec<Reconfiguration> {
        state
            .reconfigurations
            .iter()
            .map(|change| match change {
                Reconfiguration::AdditionRequested(pending) if pending == peer_id => {
                    Reconfiguration::AdditionAcknowledged(pending.clone())
                }
                other => other.clone(),
            })
            .collect()
    }

    // The changes but the one naming `peer_id`, which has ended.
    #[must_use]
    pub fn ended(state: &NodeState, peer_id: &str) -> Vec<Reconfiguration> {
        state
            .reconfigurations
            .iter()
            .filter(|change| {
                !matches!(
                    change,
                    Reconfiguration::AdditionRequested(named)
                        | Reconfiguration::AdditionAcknowledged(named)
                        | Reconfiguration::RemovalRequested(named, _) if named == peer_id
                )
            })
            .cloned()
            .collect()
    }

    // The count the removal of `peer_id` in flight stated it leaves, if its
    // request stated one (§12).
    #[must_use]
    pub fn leaves(state: &NodeState, peer_id: Option<&String>) -> Option<u64> {
        state
            .reconfigurations
            .iter()
            .find_map(|change| match change {
                Reconfiguration::RemovalRequested(named, leaves) if Some(named) == peer_id => {
                    *leaves
                }
                _ => None,
            })
    }

    // The changes with `change` requested last.
    #[must_use]
    pub fn pending(state: &NodeState, change: Reconfiguration) -> Vec<Reconfiguration> {
        state
            .reconfigurations
            .iter()
            .cloned()
            .chain([change])
            .collect()
    }
}
