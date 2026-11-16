// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::ToString;
use serde_json::Value;

use crate::command::Command;
use crate::vocabulary::result_name::ResultName;

// What the specification says of the epoch across its rows (§1, §16): a
// commit accepted at an epoch above the node's moves it there, and an epoch
// of 0 is omitted wherever it would stand.
pub struct Epoch;

impl Epoch {
    // The node's epoch after a command answered with `result`, `changed` the
    // epoch its row's change leaves.
    #[must_use]
    pub fn after(command: &Command, result: ResultName, changed: u64) -> u64 {
        if command.name.commits() && result == ResultName::Accepted {
            changed.max(command.epoch)
        } else {
            changed
        }
    }

    // A payload with the epoch among its members, unless it is 0.
    #[must_use]
    pub fn stated(mut payload: Value, epoch: u64) -> Value {
        if epoch > 0 {
            payload["epoch"] = Value::from(epoch.to_string());
        }
        payload
    }
}
