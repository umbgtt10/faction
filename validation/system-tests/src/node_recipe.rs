// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::quorum_policy::QuorumPolicy;
use faction::types::PeerId;

use faction_protocol::newcomer::Newcomer;
use faction_protocol::protocol::Protocol;
use faction_protocol::transport::Transport;

use crate::faction_node::FactionNode;
use crate::infra::logging::shared_file_observer::SharedFileObserver;
use crate::infra::logging::shared_writer::SharedWriter;
use crate::infra::timer::real::real_timer::RealTimer;
use crate::no_op_node_observer::NoOpNodeObserver;
use crate::node::Node;
use crate::node_observer::NodeObserver;
use crate::role::Role;
use crate::spawn::Spawn;
use crate::time_ports::TimePorts;

#[derive(Clone)]
pub struct NodeRecipe {
    required: usize,
    delay: Duration,
    deadline_delay: Duration,
    log_dir: Option<PathBuf>,
    time: TimePorts,
}

impl NodeRecipe {
    #[must_use]
    pub fn new(
        required: usize,
        delay: Duration,
        deadline_delay: Duration,
        log_dir: Option<PathBuf>,
        time: TimePorts,
    ) -> Self {
        Self {
            required,
            delay,
            deadline_delay,
            log_dir,
            time,
        }
    }

    #[must_use]
    pub fn build(
        &self,
        peer_id: PeerId,
        peers: Vec<PeerId>,
        transport: Box<dyn Transport>,
    ) -> FactionNode {
        let (faction_observer, node_observer) = self.observers(peer_id);
        let config = Config::new(peer_id, peers.clone(), QuorumPolicy::new(self.required));
        let protocol = Protocol::new(
            Faction::new(config, faction_observer),
            peers.clone(),
            peer_id,
        );
        FactionNode::new(
            peer_id,
            peers,
            Role::Running(Box::new(protocol)),
            transport,
            self.timer(),
            node_observer,
        )
    }

    // A newcomer that waits for its membership, connected to the transport.
    #[must_use]
    pub fn build_newcomer(&self, peer_id: PeerId, transport: Box<dyn Transport>) -> FactionNode {
        let (faction_observer, node_observer) = self.observers(peer_id);
        FactionNode::new(
            peer_id,
            Vec::new(),
            Role::Waiting(Newcomer::new(peer_id, faction_observer)),
            transport,
            self.timer(),
            node_observer,
        )
    }

    #[must_use]
    pub fn newcomer(&self, spawn: Spawn, peer_id: PeerId, transport: Box<dyn Transport>) -> Node {
        match spawn {
            Spawn::Task => Node::task(Rc::new(RefCell::new(
                self.build_newcomer(peer_id, transport),
            ))),
            Spawn::Thread => {
                let recipe = self.clone();
                Node::spawn_thread(
                    move || recipe.build_newcomer(peer_id, transport),
                    self.time.clone(),
                    self.delay,
                )
            }
            Spawn::Process => panic!("a recipe builds in-process nodes only"),
        }
    }

    #[must_use]
    pub fn node(
        &self,
        spawn: Spawn,
        peer_id: PeerId,
        peers: Vec<PeerId>,
        transport: Box<dyn Transport>,
    ) -> Node {
        match spawn {
            Spawn::Task => Node::task(Rc::new(RefCell::new(self.build(peer_id, peers, transport)))),
            Spawn::Thread => {
                let recipe = self.clone();
                Node::spawn_thread(
                    move || recipe.build(peer_id, peers, transport),
                    self.time.clone(),
                    self.delay,
                )
            }
            Spawn::Process => panic!("a recipe builds in-process nodes only"),
        }
    }

    fn observers(&self, peer_id: PeerId) -> (Box<dyn Observer>, Box<dyn NodeObserver>) {
        let writer = self.writer(peer_id);
        let faction_observer: Box<dyn Observer> = match &writer {
            Some(writer) => Box::new(SharedFileObserver::new(
                writer.clone(),
                peer_id,
                self.time.wall_clock.clone(),
            )),
            None => Box::new(NoOpObserver),
        };
        let node_observer: Box<dyn NodeObserver> = match &writer {
            Some(writer) => Box::new(SharedFileObserver::new(
                writer.clone(),
                peer_id,
                self.time.wall_clock.clone(),
            )),
            None => Box::new(NoOpNodeObserver),
        };
        (faction_observer, node_observer)
    }

    fn timer(&self) -> Box<RealTimer> {
        Box::new(RealTimer::with_delays(
            self.time.clock.clone(),
            self.delay,
            self.deadline_delay,
        ))
    }

    fn writer(&self, peer_id: PeerId) -> Option<SharedWriter> {
        self.log_dir
            .as_ref()
            .map(|dir| SharedWriter::open(&dir.join(format!("node-{peer_id}.jsonl"))))
    }
}
