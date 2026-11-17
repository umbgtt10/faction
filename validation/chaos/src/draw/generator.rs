// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_system_tests::infra::dice::Dice;

use crate::draw::change_draw::ChangeDraw;
use crate::draw::fault_draw::FaultDraw;
use crate::draw::restart_draw::RestartDraw;
use crate::model::change::Change;
use crate::model::planned_change::PlannedChange;
use crate::model::scenario::Scenario;

const CHANGE_SALT: u64 = 0x4348_414E_4745_0000;
const CHAOS_SPAN: u64 = 200;
const DUPLICATE_PERCENTS: [u8; 4] = [0, 0, 10, 30];
const FAULT_SALT: u64 = 0x4641_554C_5400_0000;
const MAX_LAG: u64 = 5;
const MAX_THRESHOLD: u64 = 3;
const MIN_CHAOS_ROUNDS: u64 = 100;
const MIN_NODES: u64 = 3;
const NODE_SPAN: u64 = 4;
const QUIESCENCE_ROUNDS: u64 = 3_000;
const RESTART_SALT: u64 = 0x5245_5354_4152_5400;
const SEED_MIX: u64 = 0x9E37_79B9_7F4A_7C15;

// The scenario a seed draws: a cluster of three to six, its required count
// and suspicion threshold, its changes, faults and crashes, and the commits'
// lateness and duplicates. The same seed draws the same scenario; the seed is
// mixed before it is drawn from, so neighbouring seeds -- which a xorshift
// would take alike, being never even -- draw scenarios of their own.
pub struct Generator {
    seed: u64,
}

impl Generator {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    #[must_use]
    pub fn scenario(&self) -> Scenario {
        let mixed = self.seed.wrapping_mul(SEED_MIX);
        let mut dice = Dice::new(mixed);
        let nodes = (MIN_NODES + dice.below(NODE_SPAN)) as usize;
        let required_count = 1 + dice.below(nodes as u64) as usize;
        let suspicion_threshold = 1 + dice.below(MAX_THRESHOLD) as usize;
        let chaos_rounds = MIN_CHAOS_ROUNDS + dice.below(CHAOS_SPAN);
        let changes =
            ChangeDraw::new(mixed ^ CHANGE_SALT).changes(nodes, required_count, chaos_rounds);
        let restartable = Self::stable(nodes, &changes);
        let faults = FaultDraw::new(mixed ^ FAULT_SALT).windows(
            &Self::peers(nodes, &changes),
            &restartable,
            chaos_rounds,
        );
        let restarts = RestartDraw::new(mixed ^ RESTART_SALT).restarts(&restartable, chaos_rounds);
        Scenario {
            seed: self.seed,
            nodes,
            required_count,
            suspicion_threshold,
            changes,
            faults,
            restarts,
            delivery_lag: dice.below(MAX_LAG + 1),
            duplicates: DUPLICATE_PERCENTS[dice.below(DUPLICATE_PERCENTS.len() as u64) as usize],
            chaos_rounds,
            quiescence_rounds: QUIESCENCE_ROUNDS,
        }
    }

    // Every peer the scenario runs: the genesis peers and the newcomers its
    // changes add.
    fn peers(nodes: usize, changes: &[PlannedChange]) -> Vec<PeerId> {
        let newcomers = changes
            .iter()
            .filter_map(|planned| match planned.change {
                Change::Addition { peer_id } if peer_id >= nodes as PeerId => Some(peer_id),
                _ => None,
            })
            .max()
            .map_or(nodes as PeerId, |last| last + 1);
        (0..newcomers).collect()
    }

    // The genesis peers no change removes.
    fn stable(nodes: usize, changes: &[PlannedChange]) -> Vec<PeerId> {
        (0..nodes as PeerId)
            .filter(|peer_id| {
                !changes
                    .iter()
                    .any(|planned| planned.change == Change::Removal { peer_id: *peer_id })
            })
            .collect()
    }
}
