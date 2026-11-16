// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The memory tier, membership changes: a concluded node adding a newcomer and
// removing it again, change after change -- nothing kept of a change once it
// ends, so a hundred hold what one does.

use crate::support::installed::ALLOCATOR;
use faction_memory::reconfiguration_sequence::ReconfigurationSequence;

#[test]
fn run_of_changes_holds_as_much_after_a_hundred_as_after_one() {
    // Arrange
    let sequence = ReconfigurationSequence::new(&ALLOCATOR, 5);

    // Act
    let profiles = [sequence.run(1), sequence.run(100)];

    // Assert
    let [one, hundred] = profiles;
    assert_eq!(
        hundred.held_after_all, one.held_after_all,
        "{hundred:?}, {one:?}"
    );
}
