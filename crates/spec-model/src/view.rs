// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use serde_json::json;
use serde_json::Value;

use crate::addition::Addition;
use crate::liveness::Liveness;
use crate::node_state::NodeState;
use crate::vocabulary::stage::Stage;
use crate::vocabulary::variant::Variant;

const ACKNOWLEDGED: &str = "Acknowledged";
const BOOTSTRAPPED: &str = "Bootstrapped";
const FRESH: &str = "Fresh";
const HELD: &str = "Held";
const LOST: &str = "Lost";
const NONE: &str = "None";
const NOT_TRACKED: &str = "NotTracked";
const PROBED: &str = "Probed";
const REQUESTED: &str = "Requested";
const TIMED_OUT: &str = "TimedOut";

// What a node shows of its state (§4).
pub struct View;

impl View {
    #[must_use]
    pub fn of(state: &NodeState) -> Value {
        json!({
            "peer_state": Variant::unit(Self::peer_state(state)),
            "is_pinging_completed": matches!(
                state.stage,
                Stage::Collecting | Stage::Bootstrapped | Stage::Reconfiguring
            ),
            "pinging_peers": state.pinging_peers,
            "collecting_peers": state.collecting_peers,
            "required_count": state.required_count.to_string(),
            "deadline_missed": state.deadline_missed,
            "members": state.members,
            "addition": Self::addition(&state.addition),
            "suspected": state.suspected,
            "misses": Self::misses(state),
            "last_probed": Self::last_probed(state.last_probed.as_ref()),
            "quorum": Variant::unit(Self::quorum(state)),
        })
    }

    fn addition(addition: &Addition) -> Value {
        match addition {
            Addition::None => Variant::unit(NONE),
            Addition::Requested(peer_id) => Variant::of(REQUESTED, json!({"peer_id": peer_id})),
            Addition::Acknowledged(peer_id) => {
                Variant::of(ACKNOWLEDGED, json!({"peer_id": peer_id}))
            }
        }
    }

    fn last_probed(last_probed: Option<&String>) -> Value {
        match last_probed {
            None => Variant::unit(NONE),
            Some(peer_id) => Variant::of(PROBED, json!({"peer_id": peer_id})),
        }
    }

    fn misses(state: &NodeState) -> Vec<Value> {
        state
            .members
            .iter()
            .filter_map(|member| {
                state
                    .misses
                    .get(member)
                    .map(|misses| json!({"peer_id": member, "misses": misses.to_string()}))
            })
            .collect()
    }

    fn peer_state(state: &NodeState) -> &'static str {
        match (state.stage, state.deadline_missed) {
            (Stage::Initial, _) => FRESH,
            (Stage::Bootstrapped | Stage::Reconfiguring, _) => BOOTSTRAPPED,
            (_, false) => state.stage.name(),
            (_, true) => TIMED_OUT,
        }
    }

    fn quorum(state: &NodeState) -> &'static str {
        match (state.stage, Liveness::holds_quorum(state)) {
            (Stage::Bootstrapped | Stage::Reconfiguring, true) => HELD,
            (Stage::Bootstrapped | Stage::Reconfiguring, false) => LOST,
            _ => NOT_TRACKED,
        }
    }
}
