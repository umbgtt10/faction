// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use faction::cluster_view::ClusterView;
use faction::command::Command;
use faction::observer::Observer;
use faction::transition::Transition;
use faction::types::PeerId;

use faction_protocol::input_message::InputMessage;
use faction_protocol::output_message::OutputMessage;

use crate::infra::logging::shared_writer::SharedWriter;
use crate::node_observer::NodeObserver;
use crate::traits::wall_clock::WallClock;

pub struct SharedFileObserver {
    writer: SharedWriter,
    peer_id: PeerId,
    clock: Arc<dyn WallClock>,
}

impl SharedFileObserver {
    #[must_use]
    pub fn new(writer: SharedWriter, peer_id: PeerId, clock: Arc<dyn WallClock>) -> Self {
        Self {
            writer,
            peer_id,
            clock,
        }
    }

    fn now_utc(&self) -> String {
        self.clock
            .now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    }
}

impl NodeObserver for SharedFileObserver {
    fn on_start(&mut self) {
        let ts = self.now_utc();
        self.writer.write_line(&format!(
            r#"{{"timestamp":"{ts}","peer_id":{},"event":"start"}}"#,
            self.peer_id
        ));
    }

    fn on_step(&mut self, input: &InputMessage, decisions: &[OutputMessage]) {
        let ts = self.now_utc();
        self.writer.write_line(&format!(
                r#"{{"timestamp":"{ts}","peer_id":{},"event":"step","input":"{input:?}","decisions":{decisions:?}}}"#,
                self.peer_id,
            ),
        );
    }

    fn on_idle(&mut self) {}
}

impl Observer for SharedFileObserver {
    fn observe(&mut self, command: Command, transition: Transition) {
        let ts = self.now_utc();
        self.writer.write_line(&format!(
                r#"{{"timestamp":"{ts}","peer_id":{},"event":"accepted","command":"{command:?}","peer_state":"{:?}","is_exited":{},"pinging_peers":{},"collecting_peers":{}}}"#,
                self.peer_id,
                transition.new_view.peer_state(),
                transition.new_view.is_concluded(),
                transition.new_view.pinging_peers().len(),
                transition.new_view.collecting_peers().len(),
            ),
        );
    }

    fn observe_query(&mut self, command: Command, cluster_view: ClusterView) {
        let ts = self.now_utc();
        self.writer.write_line(&format!(
                r#"{{"timestamp":"{ts}","peer_id":{},"event":"query","command":"{command:?}","peer_state":"{:?}","is_exited":{},"pinging_peers":{},"collecting_peers":{}}}"#,
                self.peer_id,
                cluster_view.peer_state(),
                cluster_view.is_concluded(),
                cluster_view.pinging_peers().len(),
                cluster_view.collecting_peers().len(),
            ),
        );
    }

    fn observe_rejection(
        &mut self,
        command: Command,
        cluster_view: ClusterView,
        _admissible: Vec<Command>,
    ) {
        let ts = self.now_utc();
        self.writer.write_line(&format!(
                r#"{{"timestamp":"{ts}","peer_id":{},"event":"rejected","command":"{command:?}","peer_state":"{:?}","is_exited":{},"pinging_peers":{},"collecting_peers":{}}}"#,
                self.peer_id,
                cluster_view.peer_state(),
                cluster_view.is_concluded(),
                cluster_view.pinging_peers().len(),
                cluster_view.collecting_peers().len(),
            ),
        );
    }
}
