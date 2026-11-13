// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::thread::sleep;
use std::time::Duration;

use crate::traits::pause::Pause;

pub struct ThreadPause;

impl Pause for ThreadPause {
    fn pause(&self, duration: Duration) {
        sleep(duration);
    }
}
