// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::traits::monotonic_clock_tests::FakeMonotonicClock;
use crate::traits::pause_tests::FakePause;
use crate::traits::wall_clock_tests::FakeWallClock;
use faction::types::PeerId;
use faction_system_tests::faction_node::FactionNode;
use faction_system_tests::infra::timer::real::std_monotonic_clock::StdMonotonicClock;
use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::infra::timer::real::thread_pause::ThreadPause;
use faction_system_tests::infra::transport::in_memory::in_memory_transport::InMemoryTransport;
use faction_system_tests::node_recipe::NodeRecipe;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::traits::wall_clock::WallClock;
use std::env::var;
use std::io::Result as IoResult;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;

const LOG_RUN: &str = "FACTION_LOG_RUN";

pub fn fake_time() -> (TimePorts, Arc<FakeMonotonicClock>) {
    let clock = Arc::new(FakeMonotonicClock::at(Duration::ZERO));
    let time = TimePorts {
        clock: clock.clone(),
        pause: Arc::new(FakePause::advancing(clock.clone())),
        wall_clock: Arc::new(FakeWallClock::at(2026, 11, 12)),
    };
    (time, clock)
}

pub fn in_memory_node(peers: &[PeerId], required: usize, time: &TimePorts) -> FactionNode {
    let transport = InMemoryTransport::new_mesh(peers).remove(0);
    recipe(required, time).build(peers[0], peers.to_vec(), Box::new(transport))
}

pub fn log_dir(label: &str) -> PathBuf {
    logs_root().join(run_id()).join(label.to_lowercase())
}

fn logs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../logs")
}

pub fn real_time() -> TimePorts {
    TimePorts {
        clock: Arc::new(StdMonotonicClock::new()),
        pause: Arc::new(ThreadPause),
        wall_clock: Arc::new(SystemWallClock),
    }
}

pub fn recipe(required: usize, time: &TimePorts) -> NodeRecipe {
    NodeRecipe::new(
        required,
        1,
        Duration::from_millis(10),
        Duration::from_secs(10),
        None,
        time.clone(),
    )
}

// The run a test's logs belong to: the system tests' run, whose folder under
// logs/ the xtask names and prunes; run outside it, this process's own run,
// named the same way so the same pruning keeps it in bounds.
fn run_id() -> String {
    static PROCESS_RUN: OnceLock<String> = OnceLock::new();
    var(LOG_RUN).unwrap_or_else(|_| {
        PROCESS_RUN
            .get_or_init(|| {
                SystemWallClock
                    .now()
                    .format("%Y-%m-%d_%H-%M-%S")
                    .to_string()
            })
            .clone()
    })
}

#[derive(Clone, Default)]
pub struct SharedOutput {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl SharedOutput {
    pub fn text(&self) -> String {
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
