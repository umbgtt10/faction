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

pub struct SharedFileObserver {
    writer: Arc<Mutex<BufWriter<File>>>,
    peer_id: PeerId,
}

impl SharedFileObserver {
    #[must_use]
    pub fn new(writer: Arc<Mutex<BufWriter<File>>>, peer_id: PeerId) -> Self {
        Self { writer, peer_id }
    }

    fn write_line(&mut self, event: &str, command: &Command, cluster_view: &ClusterView) {
        let line = format!(
            r#"{{"peer_id":{},"event":"{}","command":"{command:?}","peer_state":"{:?}","is_exited":{},"pinging_peers":{},"collecting_peers":{}}}"#,
            self.peer_id,
            event,
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

impl Observer for SharedFileObserver {
    fn observe(&mut self, command: Command, transition: Transition) {
        self.write_line("accepted", &command, &transition.new_view());
    }

    fn observe_query(&mut self, command: Command, cluster_view: ClusterView) {
        self.write_line("query", &command, &cluster_view);
    }

    fn observe_rejection(
        &mut self,
        command: Command,
        cluster_view: ClusterView,
        _admissible: Vec<Command>,
    ) {
        self.write_line("rejected", &command, &cluster_view);
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
