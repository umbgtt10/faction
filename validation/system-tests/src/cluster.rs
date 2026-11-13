// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::write;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use faction::peer_state::PeerState;
use faction::reconfiguration::Reconfiguration;
use faction::types::PeerId;

use crate::approver::Approver;
use crate::committer::Committer;
use crate::join::joining::Joining;
use crate::node::Node;
use crate::node_snapshot::NodeSnapshot;
use crate::spawn::Spawn;
use crate::timer_delay::TimerDelay;
use crate::traits::pause::Pause;

pub struct Cluster {
    nodes: Vec<Node>,
    peer_ids: Vec<PeerId>,
    spawn: Spawn,
    poll_delay: Duration,
    started: bool,
    joining: Option<Joining>,
    log_dir: Option<PathBuf>,
    pause: Arc<dyn Pause>,
    added_addrs: Vec<(PeerId, SocketAddr)>,
}

impl Cluster {
    #[must_use]
    pub fn new(
        nodes: Vec<Node>,
        peer_ids: Vec<PeerId>,
        spawn: Spawn,
        timer_delay: TimerDelay,
        joining: Option<Joining>,
        log_dir: Option<PathBuf>,
        pause: Arc<dyn Pause>,
    ) -> Self {
        Self {
            nodes,
            peer_ids,
            spawn,
            poll_delay: timer_delay.duration(),
            started: false,
            joining,
            log_dir,
            pause,
            added_addrs: Vec::new(),
        }
    }

    pub fn abort_addition(&self, coordinator: usize, newcomer_id: PeerId) {
        self.nodes[coordinator].abort_addition(newcomer_id);
    }

    // An addition carried through: the newcomer added, its addition requested
    // of the coordinator and acknowledged, then committed or aborted as the
    // committing layer -- the harness -- decides.
    pub fn add(&mut self, coordinator: usize, newcomer_id: PeerId, committer: Committer) {
        self.add_newcomer(newcomer_id);
        self.request_addition(coordinator, newcomer_id);
        self.poll_until_proposed(coordinator, newcomer_id);
        match committer {
            Committer::Commit { required_count } => {
                self.commit_addition(newcomer_id, required_count);
            }
            Committer::Abort => self.abort_addition(coordinator, newcomer_id),
        }
    }

    // A newcomer started and waiting for its membership, reachable by every
    // node and reaching every node it can know the address of.
    pub fn add_newcomer(&mut self, newcomer_id: PeerId) {
        let (newcomer, newcomer_addr) = match &self.joining {
            Some(Joining::Process(context)) => {
                let addr = context.ports.addr(self.nodes.len());
                let known: Vec<(PeerId, SocketAddr)> = context
                    .genesis_addrs
                    .iter()
                    .chain(&self.added_addrs)
                    .copied()
                    .collect();
                (
                    context.spec.launch_newcomer(newcomer_id, &known, addr),
                    Some(addr),
                )
            }
            Some(Joining::InProcess(context)) => (
                context
                    .recipe
                    .newcomer(self.spawn, newcomer_id, context.mesh.connect(newcomer_id)),
                None,
            ),
            None => panic!("an addition needs a cluster that can be joined"),
        };
        newcomer.start();
        if let Some(addr) = newcomer_addr {
            for node in &self.nodes {
                node.add_peer_address(newcomer_id, addr);
            }
            self.added_addrs.push((newcomer_id, addr));
        }
        self.nodes.push(newcomer);
        self.peer_ids.push(newcomer_id);
    }

    pub fn admit(&self, peer_id: PeerId) {
        for node in &self.nodes {
            node.admit(peer_id);
        }
    }

    // The commit reported to every node, the newcomer first -- the members
    // ignore it until they hold the commit, so it never concludes on the old
    // count -- with the new required count, if one is given.
    pub fn commit_addition(&self, newcomer_id: PeerId, required_count: Option<usize>) {
        for node in self.nodes.iter().rev() {
            node.commit_addition(newcomer_id);
            if let Some(required_count) = required_count {
                node.change_required_count(required_count);
            }
        }
    }

    // The link between `first` and `second` cut, at both ends.
    pub fn cut(&self, first: PeerId, second: PeerId) {
        self.node_of(first).cut(second);
        self.node_of(second).cut(first);
    }

    pub fn expire_deadline(&self) {
        for node in &self.nodes {
            node.expire_deadline();
        }
    }

    #[must_use]
    pub fn is_bootstrapped(&self) -> bool {
        self.nodes
            .iter()
            .all(|node| node.peer_state() == PeerState::Bootstrapped)
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
        self.peer_ids.push(newcomer_id);
    }

    #[must_use]
    pub fn member_count(&self, index: usize) -> usize {
        self.nodes[index].member_count()
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn node_state(&self, index: usize) -> PeerState {
        self.nodes[index].peer_state()
    }

    // Until the cluster's nodes show what `holds` asks of them.
    pub fn poll_until(&mut self, holds: impl Fn(&Self) -> bool) {
        self.start_all();
        while !holds(self) {
            self.step_all();
            self.pause.pause(self.poll_delay);
        }
    }

    pub fn poll_until_bootstrapped(&mut self) {
        self.start_all();
        while !self.is_bootstrapped() {
            self.step_all();
            self.pause.pause(self.poll_delay);
        }
    }

    // Until the coordinator holds the newcomer's acknowledgement: its addition
    // is proposed to the layer that commits.
    pub fn poll_until_proposed(&mut self, coordinator: usize, newcomer_id: PeerId) {
        self.start_all();
        let proposed = Reconfiguration::AdditionAcknowledged {
            peer_id: newcomer_id,
        };
        while self.nodes[coordinator].snapshot().addition != proposed {
            self.step_all();
            self.pause.pause(self.poll_delay);
        }
    }

    pub fn request_addition(&self, coordinator: usize, newcomer_id: PeerId) {
        self.nodes[coordinator].request_addition(newcomer_id);
    }

    pub fn settle(&mut self, rounds: usize) {
        self.start_all();
        for _ in 0..rounds {
            self.step_all();
            self.pause.pause(self.poll_delay);
        }
    }

    // Every link of `peer_id`'s cut: it hears no one, and no one hears it.
    pub fn silence(&self, peer_id: PeerId) {
        for other in self.others(peer_id) {
            self.cut(peer_id, other);
        }
    }

    #[must_use]
    pub fn snapshot(&self, index: usize) -> NodeSnapshot {
        self.nodes[index].snapshot()
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

    // Every link of `peer_id`'s mended, as cut by `silence`.
    pub fn unsilence(&self, peer_id: PeerId) {
        for other in self.others(peer_id) {
            self.node_of(peer_id).mend(other);
            self.node_of(other).mend(peer_id);
        }
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
        context
            .recipe
            .node(self.spawn, newcomer_id, peers, transport)
    }

    fn build_process_newcomer(&self, newcomer_id: PeerId) -> (Node, Option<SocketAddr>) {
        let context = match &self.joining {
            Some(Joining::Process(context)) => context,
            _ => panic!("process join requires a process join context"),
        };

        let newcomer_addr = context.ports.addr(self.nodes.len());

        let mut peers: Vec<PeerId> = context.genesis_addrs.iter().map(|(id, _)| *id).collect();
        if !peers.contains(&newcomer_id) {
            peers.push(newcomer_id);
        }

        let newcomer =
            context
                .spec
                .launch(newcomer_id, &peers, &context.genesis_addrs, newcomer_addr);
        (newcomer, Some(newcomer_addr))
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

    fn node_of(&self, peer_id: PeerId) -> &Node {
        let index = self
            .peer_ids
            .iter()
            .position(|id| *id == peer_id)
            .expect("a node of the cluster");
        &self.nodes[index]
    }

    fn others(&self, peer_id: PeerId) -> Vec<PeerId> {
        self.peer_ids
            .iter()
            .copied()
            .filter(|other| *other != peer_id)
            .collect()
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        for node in &mut self.nodes {
            node.shutdown();
        }
        self.nodes.clear();
        if let Some(dir) = &self.log_dir {
            Self::consolidate(dir);
        }
    }
}
