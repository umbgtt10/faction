// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::generated_scenarios::GeneratedScenarios;
use faction_chaos::engine::real_list::RealList;
use faction_system_tests::infra::timer::real::std_monotonic_clock::StdMonotonicClock;
use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::infra::timer::real::thread_pause::ThreadPause;
use faction_system_tests::time_ports::TimePorts;
use std::env::temp_dir;
use std::io::Result as IoResult;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Default)]
struct SharedOutput {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl SharedOutput {
    fn text(&self) -> String {
        String::from_utf8(self.bytes.lock().unwrap().clone()).unwrap()
    }
}

impl Write for SharedOutput {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.bytes.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

#[test]
fn run_of_no_seeds_plays_nothing_and_says_so() {
    // Arrange
    let output = SharedOutput::default();
    let list = RealList::new(
        Vec::new(),
        Box::new(GeneratedScenarios),
        temp_dir(),
        TimePorts {
            clock: Arc::new(StdMonotonicClock::new()),
            pause: Arc::new(ThreadPause),
            wall_clock: Arc::new(SystemWallClock),
        },
        Box::new(output.clone()),
    );

    // Act
    let failed = list.run();

    // Assert
    assert_eq!(failed, 0);
    assert_eq!(output.text(), "real engine: 0 runs, 0 failed\n");
}
