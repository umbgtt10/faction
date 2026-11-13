// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::ToString;
use serde_json::json;
use serde_json::Value;

use crate::addition::Addition;
use crate::node_state::NodeState;
use crate::vocabulary::stage::Stage;
use crate::vocabulary::variant::Variant;

const ACKNOWLEDGED: &str = "Acknowledged";
const BOOTSTRAPPED: &str = "Bootstrapped";
const FRESH: &str = "Fresh";
const NONE: &str = "None";
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

    fn peer_state(state: &NodeState) -> &'static str {
        match (state.stage, state.deadline_missed) {
            (Stage::Initial, _) => FRESH,
            (Stage::Bootstrapped | Stage::Reconfiguring, _) => BOOTSTRAPPED,
            (_, false) => state.stage.name(),
            (_, true) => TIMED_OUT,
        }
    }
}
