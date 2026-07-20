// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;
use faction::types::PeerId;

use faction_protocol::protocol::Protocol;
use faction_protocol::timer_trait::Timer;

use crate::approver::Approver;
use crate::faction_node::FactionNode;
use crate::no_op_node_observer::NoOpNodeObserver;
use crate::node::Node;
use crate::spawn::Spawn;
use crate::timer::real::real_timer::RealTimer;
use crate::timer_delay::TimerDelay;
use crate::transport::in_memory::in_memory_transport::InMemoryTransport;
use crate::transport::in_memory::in_memory_transport::Registry;

pub struct JoinContext {
    registry: Registry,
    genesis_peers: Vec<PeerId>,
    node_required: usize,
    delay: Duration,
}

impl JoinContext {
    #[must_use]
    pub fn new(
        registry: Registry,
        genesis_peers: Vec<PeerId>,
        node_required: usize,
        delay: Duration,
    ) -> Self {
        Self {
            registry,
            genesis_peers,
            node_required,
            delay,
        }
    }
}

pub struct Cluster {
    nodes: Vec<Node>,
    spawn: Spawn,
    poll_delay: Duration,
    started: bool,
    join_context: Option<JoinContext>,
}

impl Cluster {
    #[must_use]
    pub fn new(
        nodes: Vec<Node>,
        spawn: Spawn,
        timer_delay: TimerDelay,
        join_context: Option<JoinContext>,
    ) -> Self {
        Self {
            nodes,
            spawn,
            poll_delay: timer_delay.duration(),
            started: false,
            join_context,
        }
    }

    pub fn start_all(&mut self) {
        if self.started {
            return;
        }
        for node in &self.nodes {
            node.start();
        }
        self.started = true;
    }

    pub fn step_all(&mut self) {
        for node in &self.nodes {
            node.step();
        }
    }

    #[must_use]
    pub fn is_bootstrapped(&self) -> bool {
        self.nodes
            .iter()
            .all(|node| node.peer_state() == PeerState::Bootstrapped)
    }

    pub fn poll_until_bootstrapped(&mut self) {
        self.start_all();
        if matches!(self.spawn, Spawn::Process) {
            for node in &mut self.nodes {
                node.wait();
            }
        } else {
            while !self.is_bootstrapped() {
                self.step_all();
                sleep(self.poll_delay);
            }
        }
    }

    pub fn join(&mut self, newcomer_id: PeerId, approver: Approver) {
        let newcomer = self.build_newcomer(newcomer_id);
        newcomer.start();

        for member in &self.nodes {
            member.request_join(newcomer_id);
            match approver {
                Approver::AcceptAll => member.admit(newcomer_id),
                Approver::RejectAll => member.deny(newcomer_id),
            }
        }

        self.nodes.push(newcomer);
    }

    fn build_newcomer(&self, newcomer_id: PeerId) -> Node {
        let context = self
            .join_context
            .as_ref()
            .expect("join is only supported for a Task x In-Memory cluster");

        let mut peers = context.genesis_peers.clone();
        if !peers.contains(&newcomer_id) {
            peers.push(newcomer_id);
        }

        let transport = InMemoryTransport::join_mesh(newcomer_id, context.registry.clone());
        let config = Config::new(
            newcomer_id,
            peers.clone(),
            QuorumPolicy::new(context.node_required),
        );
        let protocol = Protocol::new(
            Faction::new(config, Box::new(NoOpObserver)),
            peers.clone(),
            newcomer_id,
        );
        let timer: Box<dyn Timer> = Box::new(RealTimer::with_delay(context.delay));
        let faction_node = FactionNode::new(
            newcomer_id,
            peers,
            protocol,
            Box::new(transport),
            timer,
            Box::new(NoOpNodeObserver),
            context.delay,
        );
        Node::task(Rc::new(RefCell::new(faction_node)))
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        for node in &mut self.nodes {
            node.shutdown();
        }
    }
}
