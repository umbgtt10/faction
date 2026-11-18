// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::constants::FIRST_INCARNATION;
use crate::constants::HELPERS;
use crate::infra::coordination::Coordination;
use crate::infra::fault_board::FaultBoard;
use crate::infra::invariants::breaches::Breaches;
use crate::infra::invariants::invariant_observer::InvariantObserver;
use crate::infra::node_faults::NodeFaults;
use crate::infra::timer::faulty::faulty_clock::FaultyClock;
use crate::infra::timer::faulty::faulty_timer::FaultyTimer;
use crate::infra::timer::timer_delays::TimerDelays;
use crate::infra::transport::faulty::faulty_transport::FaultyTransport;
use faction::instance::instance_id::InstanceId;
use std::cell::RefCell;
use std::io::stderr;
use std::rc::Rc;
use std::sync::Arc;

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
use crate::infra::logging::banner_fault_report::BannerFaultReport;
use crate::infra::logging::logs::Logs;
use crate::infra::logging::no_op_fault_report::NoOpFaultReport;
use crate::infra::logging::shared_file_fault_report::SharedFileFaultReport;
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
use crate::traits::fault_report::FaultReport;
use crate::traits::roles::Roles;

#[derive(Clone)]
pub struct NodeRecipe {
    required: usize,
    coordination: Coordination,
    delays: TimerDelays,
    logs: Logs,
    time: TimePorts,
    faults: FaultBoard,
    breaches: Breaches,
}

impl NodeRecipe {
    // `faults`: the board each node it builds reads its faults from;
    // `breaches`: the board its nodes' invariant observers report into.
    #[must_use]
    pub fn new(
        required: usize,
        coordination: Coordination,
        delays: TimerDelays,
        logs: Logs,
        time: TimePorts,
        faults: FaultBoard,
        breaches: Breaches,
    ) -> Self {
        Self {
            required,
            coordination,
            delays,
            logs,
            time,
            faults,
            breaches,
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
        let instance = Self::first(peer_id);
        self.node_of(instance, self.running(instance, &genesis), transport)
    }

    #[must_use]
    pub fn build_newcomer(&self, peer_id: PeerId, transport: Box<dyn Transport>) -> FactionNode {
        let instance = Self::first(peer_id);
        self.node_of(instance, self.waiting(instance), transport)
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
                    self.delays.delay,
                    self.delays.lifetime,
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
                    self.delays.delay,
                    self.delays.lifetime,
                )
            }
            Spawn::Process => panic!("a recipe builds in-process nodes only"),
        }
    }

    // The node's transport behind the faults a test injects: those its board
    // holds for it, the links its cuts name cut, every fault announced as its
    // logs say.
    fn faulty(
        &self,
        peer_id: PeerId,
        transport: Box<dyn Transport>,
        faults: &NodeFaults,
    ) -> Box<dyn Transport> {
        Box::new(FaultyTransport::new(
            peer_id,
            faults.transport.clone(),
            faults.cuts.clone(),
            self.report(peer_id),
            transport,
        ))
    }

    // A node's first instance.
    fn first(peer_id: PeerId) -> InstanceId {
        InstanceId {
            node: peer_id,
            incarnation: FIRST_INCARNATION,
        }
    }

    // A node of `role`, its roles made anew by this recipe.
    fn node_of(
        &self,
        instance: InstanceId,
        role: Role,
        transport: Box<dyn Transport>,
    ) -> FactionNode {
        let peer_id = instance.node;
        let (_, node_observer) = self.observers(instance);
        let faults = self.faults.node(peer_id);
        FactionNode::new(
            instance,
            role,
            self.faulty(peer_id, transport, &faults),
            self.timer(peer_id, &faults),
            node_observer,
            faults,
            Box::new(self.clone()),
        )
    }

    fn observers(&self, instance: InstanceId) -> (Box<dyn Observer>, Box<dyn NodeObserver>) {
        let writer = self.writer(instance.node);
        let faction_observer: Box<dyn Observer> = match &writer {
            Some(writer) => Box::new(SharedFileObserver::new(
                writer.clone(),
                instance,
                self.time.wall_clock.clone(),
            )),
            None => Box::new(NoOpObserver),
        };
        let node_observer: Box<dyn NodeObserver> = match &writer {
            Some(writer) => Box::new(SharedFileObserver::new(
                writer.clone(),
                instance,
                self.time.wall_clock.clone(),
            )),
            None => Box::new(NoOpNodeObserver),
        };
        (faction_observer, node_observer)
    }

    // Where the node announces the faults it injects: its log, a banner on
    // stderr, or nowhere.
    fn report(&self, peer_id: PeerId) -> Box<dyn FaultReport> {
        match (&self.logs, self.writer(peer_id)) {
            (_, Some(writer)) => Box::new(SharedFileFaultReport::new(
                writer,
                self.time.wall_clock.clone(),
            )),
            (Logs::Silent, None) => Box::new(NoOpFaultReport),
            _ => Box::new(BannerFaultReport::new(Box::new(stderr()))),
        }
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
                self.delays.delay,
                self.delays.deadline_delay,
                self.delays.probe_stretch,
            )),
        ))
    }

    fn writer(&self, peer_id: PeerId) -> Option<SharedWriter> {
        match &self.logs {
            Logs::Files(dir) => Some(SharedWriter::open(
                &dir.join(format!("node-{peer_id}.jsonl")),
            )),
            Logs::Stderr | Logs::Silent => None,
        }
    }
}

// A recipe's roles: a node created at a membership, or a newcomer, each with
// a faction observer of its own on the node's log, behind the invariant
// observer that reports its breaches.
impl Roles for NodeRecipe {
    fn running(&self, instance: InstanceId, membership: &Membership) -> Role {
        let peer_id = instance.node;
        let (faction_observer, _) = self.observers(instance);
        let watched = InvariantObserver::new(
            peer_id,
            Some(membership.peers.clone()),
            membership.epoch,
            self.breaches.clone(),
            faction_observer,
        );
        let config = Config::new(
            instance,
            membership.peers.clone(),
            QuorumPolicy::new(membership.required_count),
            self.coordination.suspicion_threshold(),
            membership.epoch,
            false,
            self.coordination.bound(peer_id),
        );
        Role::Running(Box::new(Protocol::new(
            Faction::new(config, Box::new(watched)),
            membership.peers.clone(),
            peer_id,
            HELPERS,
        )))
    }

    fn waiting(&self, instance: InstanceId) -> Role {
        let (faction_observer, _) = self.observers(instance);
        Role::Waiting(Newcomer::new(
            instance,
            self.coordination.suspicion_threshold(),
            self.coordination.bound(instance.node),
            HELPERS,
            Box::new(InvariantObserver::new(
                instance.node,
                None,
                0,
                self.breaches.clone(),
                faction_observer,
            )),
        ))
    }
}
