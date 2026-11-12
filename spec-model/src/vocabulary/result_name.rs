// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;
use serde_json::json;
use serde_json::Value;

use crate::vocabulary::variant::Variant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultName {
    Accepted,
    Rejected,
    Probed,
}

impl ResultName {
    pub const ALL: [ResultName; 3] = [
        ResultName::Accepted,
        ResultName::Rejected,
        ResultName::Probed,
    ];

    #[must_use]
    pub fn of(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|result| result.name() == name)
            .expect("a result")
    }

    #[must_use]
    pub fn encoded(self, outcomes: Vec<Value>, cluster_view: Value, admissible: Value) -> Value {
        let carried = match self {
            ResultName::Accepted => json!({
                "outcomes": outcomes,
                "cluster_view": cluster_view,
                "admissible": admissible,
            }),
            ResultName::Rejected | ResultName::Probed => json!({
                "cluster_view": cluster_view,
                "admissible": admissible,
            }),
        };
        Variant::of(self.name(), carried)
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ResultName::Accepted => "Accepted",
            ResultName::Rejected => "Rejected",
            ResultName::Probed => "Probed",
        }
    }
}
