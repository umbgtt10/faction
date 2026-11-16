// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::command::Command;
use faction_system_tests::infra::invariants::breach::Breach;
use faction_system_tests::infra::invariants::breaches::Breaches;

fn breach(peer_id: u64) -> Breach {
    Breach {
        peer_id,
        invariant: "the epoch fell",
        command: Command::Probe,
    }
}

#[test]
fn first_of_a_board_never_reported_to_is_none() {
    // Arrange
    let breaches = Breaches::default();

    // Act
    let first = breaches.first();

    // Assert
    assert_eq!(first, None);
}

#[test]
fn report_keeps_the_first_breach_for_every_clone() {
    // Arrange
    let breaches = Breaches::default();
    let shared = breaches.clone();

    // Act
    shared.report(breach(1));
    shared.report(breach(2));

    // Assert
    assert_eq!(breaches.first(), Some(breach(1)));
}
