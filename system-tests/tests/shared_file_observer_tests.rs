// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::observer::Observer;
use faction::peer_state::PeerState;
use faction::transition::Transition;

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;
use faction_protocol::timer_message::TimerMessage;

use faction_system_tests::node_observer::NodeObserver;
use faction_system_tests::shared_file_observer::SharedFileObserver;
use faction_system_tests::shared_file_observer::new_shared_writer;

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!("test_sfo_{}_{}.jsonl", std::process::id(), n);
        Self {
            path: std::env::temp_dir().join(name),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn reader(&self) -> Arc<Mutex<std::io::BufWriter<fs::File>>> {
        new_shared_writer(&self.path)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn make_cluster_view() -> ClusterView {
    ClusterView::new(PeerState::Pinging, false, vec![0, 3], vec![1], 4)
}

fn make_transition() -> Transition {
    Transition::new(make_cluster_view(), vec![], make_cluster_view())
}

// ---------------------------------------------------------------------------
// NodeObserver
// ---------------------------------------------------------------------------

#[test]
fn node_observer_on_start_writes_start_event() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7);

    // Act
    observer.on_start();
    std::mem::drop(observer);

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
    let mut observer = SharedFileObserver::new(writer, 7);
    let input = InputMessage::Timer(TimerMessage::RetryPing);
    let decisions = vec![
        OutputMessage::Noop,
        OutputMessage::BroadcastReady,
        OutputMessage::Schedule(faction_protocol::timer_event::TimerEvent::Fire(
            TimerMessage::RetryPing,
        )),
    ];

    // Act
    observer.on_step(&input, &decisions);
    std::mem::drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"event\":\"step\""));
    assert!(content.contains("Noop"));
    assert!(content.contains("BroadcastReady"));
    assert!(content.contains("RetryPing"));
}

#[test]
fn node_observer_on_idle_does_not_write() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7);

    // Act
    observer.on_idle();
    std::mem::drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.is_empty());
}

// ---------------------------------------------------------------------------
// Observer
// ---------------------------------------------------------------------------

#[test]
fn observer_observe_writes_accepted_event() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7);
    let command = Command::ParticipationObserved { peer_id: 3 };
    let transition = make_transition();

    // Act
    observer.observe(command, transition);
    std::mem::drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"event\":\"accepted\""));
    assert!(content.contains("\"peer_state\":\"Pinging\""));
    assert!(content.contains("\"is_exited\":false"));
    assert!(content.contains("\"pinging_peers\":2"));
    assert!(content.contains("\"collecting_peers\":1"));
}

#[test]
fn observer_observe_query_writes_query_event() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7);
    let command = Command::Probe;
    let view = make_cluster_view();

    // Act
    observer.observe_query(command, view);
    std::mem::drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"event\":\"query\""));
    assert!(content.contains("\"command\":\"Probe\""));
}

#[test]
fn observer_observe_rejection_writes_rejected_event() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7);
    let command = Command::DeadlineExpired;
    let view = make_cluster_view();
    let admissible = vec![];

    // Act
    observer.observe_rejection(command, view, admissible);
    std::mem::drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"event\":\"rejected\""));
    assert!(content.contains("DeadlineExpired"));
}

#[test]
fn accepted_writes_bootstrapped_exit_state() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 7);
    let view = ClusterView::new(PeerState::Bootstrapped, true, vec![], vec![], 4);
    let transition = Transition::new(view.clone(), vec![], view);

    // Act
    observer.observe(Command::Probe, transition);
    std::mem::drop(observer);

    // Assert
    let content = read_file(tmp.path());
    assert!(content.contains("\"peer_state\":\"Bootstrapped\""));
    assert!(content.contains("\"is_exited\":true"));
}

#[test]
fn is_valid_json() {
    // Arrange
    let tmp = TempFile::new();
    let writer = tmp.reader();
    let mut observer = SharedFileObserver::new(writer, 3);

    // Act
    observer.on_start();
    observer.observe(
        Command::Probe,
        Transition::new(make_cluster_view(), vec![], make_cluster_view()),
    );
    observer.observe_query(Command::Probe, make_cluster_view());
    observer.observe_rejection(Command::DeadlineExpired, make_cluster_view(), vec![]);
    std::mem::drop(observer);

    // Assert
    let content = read_file(tmp.path());
    for line in content.lines() {
        assert!(line.starts_with('{'), "not valid JSON: {line}");
        assert!(line.ends_with('}'), "not valid JSON: {line}");
        assert!(line.contains("\"event\""), "missing event: {line}");
    }
}

#[test]
fn new_shared_writer_creates_appending_file() {
    // Arrange
    let tmp = TempFile::new();

    // Act
    let w1 = new_shared_writer(tmp.path());
    std::mem::drop(w1);
    let w2 = new_shared_writer(tmp.path());
    std::mem::drop(w2);

    // Assert
    assert!(tmp.path().exists());
}

fn read_file(path: &Path) -> String {
    let mut f = fs::File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}
