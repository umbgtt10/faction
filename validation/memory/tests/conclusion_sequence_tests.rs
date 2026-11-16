// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The memory tier, a conclusion: a node driven from new to concluded by every
// peer's participation and readiness -- the machine holding what its peers
// need, more for more of them, nothing for the duplicates and probes met
// between the signals, and nothing once it is gone.

use crate::support::installed::ALLOCATOR;
use faction_memory::conclusion_sequence::ConclusionSequence;

#[test]
fn run_of_a_conclusion_holds_more_for_more_peers() {
    // Arrange
    let fewer = ConclusionSequence::new(&ALLOCATOR, 4, false);
    let more = ConclusionSequence::new(&ALLOCATOR, 24, false);

    // Act
    let profiles = [fewer.run(), more.run()];

    // Assert
    let [fewer, more] = profiles;
    assert!(
        fewer.held_after_all > fewer.skeleton,
        "the counter never moved: {fewer:?}"
    );
    assert!(
        more.held_after_all > fewer.held_after_all,
        "{more:?}, {fewer:?}"
    );
}

#[test]
fn run_of_a_conclusion_leaves_nothing_once_the_machine_is_gone() {
    // Arrange
    let sequence = ConclusionSequence::new(&ALLOCATOR, 7, true);

    // Act
    let profile = sequence.run();

    // Assert
    assert_eq!(profile.after_drop, 0, "{profile:?}");
}

#[test]
fn run_with_duplicates_and_probes_between_the_signals_holds_what_the_signals_alone_hold() {
    // Arrange
    let quiet = ConclusionSequence::new(&ALLOCATOR, 7, false);
    let busy = ConclusionSequence::new(&ALLOCATOR, 7, true);

    // Act
    let profiles = [quiet.run(), busy.run()];

    // Assert
    let [quiet, busy] = profiles;
    assert_eq!(
        busy.held_after_all, quiet.held_after_all,
        "{busy:?}, {quiet:?}"
    );
}
