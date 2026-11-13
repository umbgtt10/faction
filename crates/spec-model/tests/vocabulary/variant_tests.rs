// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_spec_model::vocabulary::variant::Variant;
use serde_json::json;

#[test]
fn of_a_name_and_a_payload_is_an_object_of_one_member_named_for_it() {
    // Arrange & Act
    let variant = Variant::of("ReadyObserved", json!({"peer_id": "2"}));

    // Assert
    assert_eq!(variant, json!({"ReadyObserved": {"peer_id": "2"}}));
}

#[test]
fn unit_of_a_name_carries_the_empty_object() {
    // Arrange & Act
    let variant = Variant::unit("Fresh");

    // Assert
    assert_eq!(variant, json!({"Fresh": {}}));
}
