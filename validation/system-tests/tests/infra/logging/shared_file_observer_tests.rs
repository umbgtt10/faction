// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::traits::wall_clock_tests::FakeWallClock;
use faction::cluster_view::ClusterView;
use faction::cluster_view_builder::ClusterViewBuilder;
use faction::command::Command;
use faction::observer::Observer;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::timer_event::TimerEvent;
use faction_protocol::timer_message::TimerMessage;
use faction_system_tests::infra::logging::shared_file_observer::SharedFileObserver;
use faction_system_tests::infra::logging::shared_writer::SharedWriter;
use faction_system_tests::node_observer::NodeObserver;
use std::env::temp_dir;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

fn make_accepted() -> ProcessResult {
    ProcessResult::Accepted {
        cluster_view: make_cluster_view(),
        admissible: vec![],
        outcomes: vec![],
    }
}

fn make_cluster_view() -> ClusterView {
    ClusterViewBuilder::default()
        .with_peer_state(PeerState::Pinging)
        .with_pinging_peers(vec![0, 3])
        .with_collecting_peers(vec![1])
        .with_required_count(4)
        .build()
}

fn read_file(path: &Path) -> String {
    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!("test_sfo_{}_{}.jsonl", process::id(), n);
        Self {
            path: temp_dir().join(name),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn reader(&self) -> SharedWriter {
        SharedWriter::open(&self.path)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn wall_clock() -> Arc<FakeWallClock> {
    Arc::new(FakeWallClock::at(2026, 11, 12))
}

#[test]
fn node_observer_on_idle_does_not_write() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7, wall_clock());

    // Act
    observer.on_idle();
    drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.is_empty());
}

#[test]
fn node_observer_on_start_stamps_the_wall_clock_time() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7, wall_clock());

    // Act
    observer.on_start();
    drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(
        content.contains("\"timestamp\":\"2026-11-12T12:30:45.000Z\""),
        "{content}"
    );
}

#[test]
fn node_observer_on_start_writes_start_event() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7, wall_clock());

    // Act
    observer.on_start();
    drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"event\":\"start\""));
    assert!(content.contains("\"peer_id\":7"));
}

#[test]
fn node_observer_on_step_writes_step_event_with_decision_count() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7, wall_clock());
    let input = InputMessage::Timer(TimerMessage::RetryPing);
    let decisions = vec![
        OutputMessage::Noop,
        OutputMessage::BroadcastReady,
        OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryPing)),
    ];

    // Act
    observer.on_step(&input, &decisions);
    drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"event\":\"step\""));
    assert!(content.contains("Noop"));
    assert!(content.contains("BroadcastReady"));
    assert!(content.contains("RetryPing"));
}

#[test]
fn observe_accepted_writes_accepted_event() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7, wall_clock());
    let command = Command::ParticipationObserved { peer_id: 3 };

    // Act
    observer.observe(&command, &make_accepted());
    drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"event\":\"accepted\""));
    assert!(content.contains("\"peer_state\":\"Pinging\""));
    assert!(content.contains("\"is_exited\":false"));
    assert!(content.contains("\"pinging_peers\":2"));
    assert!(content.contains("\"collecting_peers\":1"));
}

#[test]
fn observe_probed_writes_query_event() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7, wall_clock());
    let result = ProcessResult::Probed {
        cluster_view: make_cluster_view(),
        admissible: vec![],
    };

    // Act
    observer.observe(&Command::Probe, &result);
    drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"event\":\"query\""));
    assert!(content.contains("\"command\":\"Probe\""));
}

#[test]
fn observe_rejected_writes_rejected_event() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7, wall_clock());
    let result = ProcessResult::Rejected {
        cluster_view: make_cluster_view(),
        admissible: vec![],
    };

    // Act
    observer.observe(&Command::DeadlineExpired, &result);
    drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"event\":\"rejected\""));
    assert!(content.contains("DeadlineExpired"));
}

#[test]
fn observe_writes_bootstrapped_exit_state() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7, wall_clock());
    let view = ClusterViewBuilder::default()
        .with_peer_state(PeerState::Bootstrapped)
        .with_is_pinging_completed(true)
        .with_required_count(4)
        .build();
    let result = ProcessResult::Probed {
        cluster_view: view,
        admissible: vec![],
    };

    // Act
    observer.observe(&Command::Probe, &result);
    drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"peer_state\":\"Bootstrapped\""));
    assert!(content.contains("\"is_exited\":true"));
}

#[test]
fn open_creates_appending_file() {
    // Arrange
    let tmp = TempFile::new();

    // Act
    let w1 = SharedWriter::open(tmp.path());
    drop(w1);
    let w2 = SharedWriter::open(tmp.path());
    drop(w2);

    // Assert
    assert!(tmp.path().exists());
}

#[test]
fn write_json_output_is_valid() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 3, wall_clock());

    // Act
    observer.on_start();
    observer.observe(
        &Command::ParticipationObserved { peer_id: 1 },
        &make_accepted(),
    );
    observer.observe(
        &Command::Probe,
        &ProcessResult::Probed {
            cluster_view: make_cluster_view(),
            admissible: vec![],
        },
    );
    observer.observe(
        &Command::DeadlineExpired,
        &ProcessResult::Rejected {
            cluster_view: make_cluster_view(),
            admissible: vec![],
        },
    );
    drop(observer);

    // Assert
    let content = read_file(tmp.path());
    for line in content.lines() {
        assert!(line.starts_with('{'), "not valid JSON: {line}");
        assert!(line.ends_with('}'), "not valid JSON: {line}");
        assert!(line.contains("\"event\""), "missing event: {line}");
    }
}
