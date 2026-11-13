// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::probe_stretch::ProbeStretch;

#[test]
fn factor_of_generous_is_three() {
    // Arrange & Act
    let factor = ProbeStretch::Generous.factor();

    // Assert
    assert_eq!(factor, 3);
}

#[test]
fn factor_of_minimal_is_one() {
    // Arrange & Act
    let factor = ProbeStretch::Minimal.factor();

    // Assert
    assert_eq!(factor, 1);
}
