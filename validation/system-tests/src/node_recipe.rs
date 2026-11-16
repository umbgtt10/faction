// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::constants::HELPERS;
use crate::infra::fault_board::FaultBoard;
use crate::infra::node_faults::NodeFaults;
use crate::infra::timer::faulty::faulty_clock::FaultyClock;
use crate::infra::timer::faulty::faulty_timer::FaultyTimer;
use crate::infra::transport::faulty::faulty_transport::FaultyTransport;
use crate::infra::transport::faulty::link_cuts::LinkCuts;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::quorum_policy::QuorumPolicy;
use faction::types::PeerId;

use faction_protocol::newcomer::Newcomer;
use faction_protocol::protocol::Protocol;
use faction_protocol::timer::Timer;
use faction_protocol::transport::Transport;

use crate::faction_node::FactionNode;
use crate::infra::logging::shared_file_observer::SharedFileObserver;
use crate::infra::logging::shared_writer::SharedWriter;
use crate::infra::timer::real::real_timer::RealTimer;
use crate::membership::Membership;
use crate::no_op_node_observer::NoOpNodeObserver;
use crate::node::Node;
use crate::node_observer::NodeObserver;
use crate::role::Role;
use crate::spawn::Spawn;
use crate::time_ports::TimePorts;
use crate::traits::roles::Roles;

#[derive(Clone)]
pub struct NodeRecipe {
    required: usize,
    suspicion_threshold: usize,
    delay: Duration,
    deadline_delay: Duration,
    probe_stretch: u32,
    log_dir: Option<PathBuf>,
    time: TimePorts,
    faults: FaultBoard,
}

impl NodeRecipe {
    // `faults`: the board each node it builds reads its faults from.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        required: usize,
        suspicion_threshold: usize,
        delay: Duration,
        deadline_delay: Duration,
        probe_stretch: u32,
        log_dir: Option<PathBuf>,
        time: TimePorts,
        faults: FaultBoard,
    ) -> Self {
        Self {
            required,
            suspicion_threshold,
            delay,
            deadline_delay,
            probe_stretch,
            log_dir,
            time,
            faults,
        }
    }

    #[must_use]
    pub fn build(
        &self,
        peer_id: PeerId,
        peers: Vec<PeerId>,
        transport: Box<dyn Transport>,
    ) -> FactionNode {
        let genesis = Membership {
            peers: peers.clone(),
            required_count: self.required,
            epoch: 0,
        };
        self.node_of(peer_id, self.running(peer_id, &genesis), transport)
    }

    #[must_use]
    pub fn build_newcomer(&self, peer_id: PeerId, transport: Box<dyn Transport>) -> FactionNode {
        self.node_of(peer_id, self.waiting(peer_id), transport)
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

    // The node's transport behind the faults a test injects: those its board
    // holds for it, the links its cuts name cut, every fault one line of the
    // node's log.
    fn faulty(
        &self,
        peer_id: PeerId,
        transport: Box<dyn Transport>,
        cuts: &LinkCuts,
        faults: &NodeFaults,
    ) -> Box<dyn Transport> {
        Box::new(FaultyTransport::new(
            peer_id,
            faults.transport.clone(),
            cuts.clone(),
            self.writer(peer_id),
            transport,
        ))
    }

    // A node of `role`, its roles made anew by this recipe.
    fn node_of(&self, peer_id: PeerId, role: Role, transport: Box<dyn Transport>) -> FactionNode {
        let (_, node_observer) = self.observers(peer_id);
        let cuts = LinkCuts::default();
        let faults = self.faults.node(peer_id);
        FactionNode::new(
            peer_id,
            role,
            self.faulty(peer_id, transport, &cuts, &faults),
            self.timer(peer_id, &faults),
            node_observer,
            cuts,
            Box::new(self.clone()),
        )
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

    // The node's timer behind the faults its board holds for it: its firings',
    // and its clock's.
    fn timer(&self, peer_id: PeerId, faults: &NodeFaults) -> Box<dyn Timer> {
        let clock = Arc::new(FaultyClock::new(
            faults.clock.clone(),
            self.time.clock.clone(),
        ));
        Box::new(FaultyTimer::new(
            peer_id,
            faults.timer.clone(),
            Box::new(RealTimer::with_delays(
                clock,
                self.delay,
                self.deadline_delay,
                self.probe_stretch,
            )),
        ))
    }

    fn writer(&self, peer_id: PeerId) -> Option<SharedWriter> {
        self.log_dir
            .as_ref()
            .map(|dir| SharedWriter::open(&dir.join(format!("node-{peer_id}.jsonl"))))
    }
}

// A recipe's roles: a node created at a membership, or a newcomer, each with
// a faction observer of its own on the node's log.
impl Roles for NodeRecipe {
    fn running(&self, peer_id: PeerId, membership: &Membership) -> Role {
        let (faction_observer, _) = self.observers(peer_id);
        let config = Config::new(
            peer_id,
            membership.peers.clone(),
            QuorumPolicy::new(membership.required_count),
            self.suspicion_threshold,
            membership.epoch,
        );
        Role::Running(Box::new(Protocol::new(
            Faction::new(config, faction_observer),
            membership.peers.clone(),
            peer_id,
            HELPERS,
        )))
    }

    fn waiting(&self, peer_id: PeerId) -> Role {
        let (faction_observer, _) = self.observers(peer_id);
        Role::Waiting(Newcomer::new(
            peer_id,
            self.suspicion_threshold,
            HELPERS,
            faction_observer,
        ))
    }
}
