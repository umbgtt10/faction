// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use crate::time_ports::TimePorts;

const RETRY: Duration = Duration::from_millis(10);

#[derive(Clone)]
pub struct TcpReadiness {
    time: TimePorts,
    timeout: Duration,
}

impl TcpReadiness {
    #[must_use]
    pub fn new(time: TimePorts, timeout: Duration) -> Self {
        Self { time, timeout }
    }

    pub fn wait(&self, addr: SocketAddr) {
        let deadline = self.time.clock.now() + self.timeout;
        while self.time.clock.now() < deadline {
            if TcpStream::connect(addr).is_ok() {
                return;
            }
            self.time.pause.pause(RETRY);
        }
        panic!("Process TCP listener not ready within timeout");
    }
}
