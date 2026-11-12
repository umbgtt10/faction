// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::timer::real::thread_pause::ThreadPause;
use faction_system_tests::traits::pause::Pause;
use std::time::Duration;

#[test]
fn pause_of_no_time_returns() {
    // Arrange & Act
    ThreadPause.pause(Duration::ZERO);

    // Assert
    // pause returned, which a thread blocked for good never does.
}
