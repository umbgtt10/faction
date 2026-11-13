// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::ToString;
use serde_json::json;
use serde_json::Value;

use crate::node_state::NodeState;
use crate::vocabulary::stage::Stage;
use crate::vocabulary::variant::Variant;

const FRESH: &str = "Fresh";
const TIMED_OUT: &str = "TimedOut";

// What a node shows of its state (§4).
pub struct View;

impl View {
    #[must_use]
    pub fn of(state: &NodeState) -> Value {
        json!({
            "peer_state": Variant::unit(Self::peer_state(state)),
            "is_pinging_completed": matches!(state.stage, Stage::Collecting | Stage::Bootstrapped),
            "pinging_peers": state.pinging_peers,
            "collecting_peers": state.collecting_peers,
            "required_count": state.required_count.to_string(),
            "deadline_missed": state.deadline_missed,
            "members": state.members,
        })
    }

    fn peer_state(state: &NodeState) -> &'static str {
        match (state.stage, state.deadline_missed) {
            (Stage::Initial, _) => FRESH,
            (Stage::Bootstrapped, _) | (_, false) => state.stage.name(),
            (_, true) => TIMED_OUT,
        }
    }
}
