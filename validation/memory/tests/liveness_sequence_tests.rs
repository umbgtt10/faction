// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The memory tier, liveness: a concluded node through probe periods, every
// member heard, one missed, suspected and heard again -- its suspicions held
// per member, so a thousand periods hold what one does.

use crate::support::installed::ALLOCATOR;
use faction_memory::liveness_sequence::LivenessSequence;

#[test]
fn run_of_probe_periods_holds_as_much_after_a_thousand_as_after_one() {
    // Arrange
    let sequence = LivenessSequence::new(&ALLOCATOR, 5);

    // Act
    let profiles = [sequence.run(1), sequence.run(1000)];

    // Assert
    let [one, thousand] = profiles;
    assert_eq!(
        thousand.held_after_all, one.held_after_all,
        "{thousand:?}, {one:?}"
    );
}
