// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use alloc::string::String;
use serde_json::Map;
use serde_json::Value;

pub struct Variant;

impl Variant {
    #[must_use]
    pub fn of(name: &str, payload: Value) -> Value {
        Value::Object([(String::from(name), payload)].into_iter().collect())
    }

    #[must_use]
    pub fn unit(name: &str) -> Value {
        Self::of(name, Value::Object(Map::new()))
    }
}
