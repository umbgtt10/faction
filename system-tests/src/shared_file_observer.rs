// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use faction::PeerId;
use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::observer::Observer;
use faction::transition::Transition;

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;

use crate::node_observer::NodeObserver;

pub struct SharedFileObserver {
    writer: Arc<Mutex<BufWriter<File>>>,
    peer_id: PeerId,
}

impl SharedFileObserver {
    #[must_use]
    pub fn new(writer: Arc<Mutex<BufWriter<File>>>, peer_id: PeerId) -> Self {
        Self { writer, peer_id }
    }
}

impl NodeObserver for SharedFileObserver {
    fn on_start(&mut self) {
        let mut w = self.writer.lock().unwrap();
        let _ = writeln!(w, r#"{{"peer_id":{},"event":"start"}}"#, self.peer_id);
        let _ = w.flush();
    }

    fn on_step(&mut self, input: &InputMessage, decisions: &[OutputMessage]) {
        let mut w = self.writer.lock().unwrap();
        let _ = writeln!(
            w,
            r#"{{"peer_id":{},"event":"step","input":"{input:?}","decisions":{}}}"#,
            self.peer_id,
            decisions.len(),
        );
        let _ = w.flush();
    }

    fn on_idle(&mut self) {}
}

impl Observer for SharedFileObserver {
    fn observe(&mut self, command: Command, transition: Transition) {
        let line = format!(
            r#"{{"peer_id":{},"event":"accepted","command":"{command:?}","peer_state":"{:?}","is_exited":{},"pinging_peers":{},"collecting_peers":{}}}"#,
            self.peer_id,
            transition.new_view().peer_state(),
            transition.new_view().is_exited(),
            transition.new_view().pinging_peers().len(),
            transition.new_view().collecting_peers().len(),
        );
        let mut w = self.writer.lock().unwrap();
        let _ = writeln!(w, "{line}");
        let _ = w.flush();
    }

    fn observe_query(&mut self, command: Command, cluster_view: ClusterView) {
        let line = format!(
            r#"{{"peer_id":{},"event":"query","command":"{command:?}","peer_state":"{:?}","is_exited":{},"pinging_peers":{},"collecting_peers":{}}}"#,
            self.peer_id,
            cluster_view.peer_state(),
            cluster_view.is_exited(),
            cluster_view.pinging_peers().len(),
            cluster_view.collecting_peers().len(),
        );
        let mut w = self.writer.lock().unwrap();
        let _ = writeln!(w, "{line}");
        let _ = w.flush();
    }

    fn observe_rejection(
        &mut self,
        command: Command,
        cluster_view: ClusterView,
        _admissible: Vec<Command>,
    ) {
        let line = format!(
            r#"{{"peer_id":{},"event":"rejected","command":"{command:?}","peer_state":"{:?}","is_exited":{},"pinging_peers":{},"collecting_peers":{}}}"#,
            self.peer_id,
            cluster_view.peer_state(),
            cluster_view.is_exited(),
            cluster_view.pinging_peers().len(),
            cluster_view.collecting_peers().len(),
        );
        let mut w = self.writer.lock().unwrap();
        let _ = writeln!(w, "{line}");
        let _ = w.flush();
    }
}

#[must_use]
pub fn new_shared_writer(path: &Path) -> Arc<Mutex<BufWriter<File>>> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    Arc::new(Mutex::new(BufWriter::new(file)))
}
