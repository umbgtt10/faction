// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::write;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;
use faction::types::PeerId;

use faction_protocol::protocol::Protocol;
use faction_protocol::timer_trait::Timer;
use faction_protocol::transport_trait::Transport;

use crate::approver::Approver;
use crate::cluster_builder::ClusterBuilder;
use crate::faction_node::FactionNode;
use crate::join::Joining;
use crate::no_op_node_observer::NoOpNodeObserver;
use crate::node::Node;
use crate::node_observer::NodeObserver;
use crate::process_spawn::spawn_process_node;
use crate::shared_file_observer::SharedFileObserver;
use crate::shared_file_observer::SharedWriter;
use crate::spawn::Spawn;
use crate::timer::real::real_timer::RealTimer;
use crate::timer_delay::TimerDelay;
use crate::transport_kind::TransportKind;

pub struct Cluster {
    nodes: Vec<Node>,
    spawn: Spawn,
    poll_delay: Duration,
    started: bool,
    joining: Option<Joining>,
    log_dir: Option<PathBuf>,
}

impl Cluster {
    #[must_use]
    pub fn new(
        nodes: Vec<Node>,
        spawn: Spawn,
        timer_delay: TimerDelay,
        joining: Option<Joining>,
        log_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            nodes,
            spawn,
            poll_delay: timer_delay.duration(),
            started: false,
            joining,
            log_dir,
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

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn node_state(&self, index: usize) -> PeerState {
        self.nodes[index].peer_state()
    }

    #[must_use]
    pub fn member_count(&self, index: usize) -> usize {
        self.nodes[index].member_count()
    }

    pub fn admit(&self, peer_id: PeerId) {
        for node in &self.nodes {
            node.admit(peer_id);
        }
    }

    pub fn expire_deadline(&self) {
        for node in &self.nodes {
            node.expire_deadline();
        }
    }

    pub fn settle(&mut self, rounds: usize) {
        self.start_all();
        for _ in 0..rounds {
            self.step_all();
            sleep(self.poll_delay);
        }
    }

    pub fn poll_until_bootstrapped(&mut self) {
        self.start_all();
        while !self.is_bootstrapped() {
            self.step_all();
            sleep(self.poll_delay);
        }
    }

    pub fn join(&mut self, newcomer_id: PeerId, approver: Approver) {
        let (newcomer, newcomer_addr) = match &self.joining {
            Some(Joining::Process(_)) => self.build_process_newcomer(newcomer_id),
            _ => (self.build_newcomer(newcomer_id), None),
        };
        newcomer.start();

        for member in &self.nodes {
            if let Some(addr) = newcomer_addr {
                member.add_peer_address(newcomer_id, addr);
            }
            member.request_join(newcomer_id);
            match approver {
                Approver::AcceptAll => member.admit(newcomer_id),
                Approver::RejectAll => member.deny(newcomer_id),
            }
        }

        self.nodes.push(newcomer);
    }

    fn build_newcomer(&self, newcomer_id: PeerId) -> Node {
        let context = match &self.joining {
            Some(Joining::InProcess(context)) => context,
            _ => panic!("join is only supported on Task or Thread clusters"),
        };

        let mut peers = context.genesis_peers.clone();
        if !peers.contains(&newcomer_id) {
            peers.push(newcomer_id);
        }

        let transport = context.mesh.connect(newcomer_id);
        let required = context.node_required;
        let delay = context.delay;
        let writer = ClusterBuilder::node_writer(&context.log_dir, newcomer_id);

        match self.spawn {
            Spawn::Task => {
                let faction_node =
                    build_faction_node(newcomer_id, peers, transport, required, delay, &writer);
                Node::task(Rc::new(RefCell::new(faction_node)))
            }
            Spawn::Thread => Node::spawn_thread(move || {
                build_faction_node(newcomer_id, peers, transport, required, delay, &writer)
            }),
            Spawn::Process => {
                unreachable!("join builds newcomers only for Task and Thread clusters")
            }
        }
    }

    fn build_process_newcomer(&self, newcomer_id: PeerId) -> (Node, Option<SocketAddr>) {
        let context = match &self.joining {
            Some(Joining::Process(context)) => context,
            _ => panic!("process join requires a process join context"),
        };

        let newcomer_addr = ClusterBuilder::allocate_port_addr(self.nodes.len());

        let mut peers: Vec<PeerId> = context.genesis_addrs.iter().map(|(id, _)| *id).collect();
        if !peers.contains(&newcomer_id) {
            peers.push(newcomer_id);
        }

        let child = spawn_process_node(
            &context.spec,
            newcomer_id,
            &peers,
            &context.genesis_addrs,
            newcomer_addr,
        );
        if matches!(context.spec.transport, TransportKind::Tcp) {
            ClusterBuilder::wait_for_tcp_ready(newcomer_addr, Duration::from_secs(30));
        }

        (Node::process(child), Some(newcomer_addr))
    }
}

fn build_faction_node(
    peer_id: PeerId,
    peers: Vec<PeerId>,
    transport: Box<dyn Transport>,
    required: usize,
    delay: Duration,
    writer: &Option<SharedWriter>,
) -> FactionNode {
    let config = Config::new(peer_id, peers.clone(), QuorumPolicy::new(required));
    let faction_observer: Box<dyn Observer> = match writer {
        Some(writer) => Box::new(SharedFileObserver::new(writer.clone(), peer_id)),
        None => Box::new(NoOpObserver),
    };
    let node_observer: Box<dyn NodeObserver> = match writer {
        Some(writer) => Box::new(SharedFileObserver::new(writer.clone(), peer_id)),
        None => Box::new(NoOpNodeObserver),
    };
    let protocol = Protocol::new(
        Faction::new(config, faction_observer),
        peers.clone(),
        peer_id,
    );
    let timer: Box<dyn Timer> = Box::new(RealTimer::with_delay(delay));
    FactionNode::new(
        peer_id,
        peers,
        protocol,
        transport,
        timer,
        node_observer,
        delay,
    )
}

impl Drop for Cluster {
    fn drop(&mut self) {
        for node in &mut self.nodes {
            node.shutdown();
        }
        self.nodes.clear();
        if let Some(dir) = &self.log_dir {
            consolidate(dir);
        }
    }
}

fn consolidate(dir: &Path) {
    let Ok(entries) = read_dir(dir) else {
        return;
    };
    let mut lines: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let is_node_log = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("node-") && name.ends_with(".jsonl"));
        if is_node_log {
            if let Ok(content) = read_to_string(&path) {
                lines.extend(content.lines().map(String::from));
            }
        }
    }
    lines.sort();
    let mut body = lines.join("\n");
    if !body.is_empty() {
        body.push('\n');
    }
    let _ = write(dir.join("consolidated.jsonl"), body);
}
