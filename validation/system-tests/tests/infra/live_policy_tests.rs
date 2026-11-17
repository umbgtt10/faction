// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::live_policy::LivePolicy;
use faction_system_tests::infra::timer::faulty::timer_fault_policy::TimerFaultPolicy;

#[test]
fn current_of_a_new_policy_reads_what_it_was_given() {
    // Arrange
    let given = TimerFaultPolicy {
        late: 30,
        ..TimerFaultPolicy::default()
    };
    let live = LivePolicy::new(given);

    // Act
    let current = live.current();

    // Assert
    assert_eq!(current, given);
}

#[test]
fn set_on_a_clone_changes_what_every_clone_reads() {
    // Arrange
    let live = LivePolicy::new(TimerFaultPolicy::default());
    let shared = live.clone();
    let dead = TimerFaultPolicy {
        dead: true,
        ..TimerFaultPolicy::default()
    };

    // Act
    shared.set(dead);

    // Assert
    assert_eq!(live.current(), dead);
}
