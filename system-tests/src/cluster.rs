// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::net::SocketAddr;
use std::net::TcpListener;
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
use crate::cluster_builder::ProcessSpec;
use crate::cluster_builder::spawn_process_node;
use crate::cluster_builder::wait_for_tcp_ready;
use crate::faction_node::FactionNode;
use crate::no_op_node_observer::NoOpNodeObserver;
use crate::node::Node;
use crate::node_observer::NodeObserver;
use crate::shared_file_observer::SharedFileObserver;
use crate::shared_file_observer::SharedWriter;
use crate::spawn::Spawn;
use crate::timer::real::real_timer::RealTimer;
use crate::timer_delay::TimerDelay;
use crate::transport::channels::channels_transport::ChannelRegistry;
use crate::transport::channels::channels_transport::ChannelsTransport;
use crate::transport::grpc::grpc_transport::AddressBook as GrpcAddressBook;
use crate::transport::grpc::grpc_transport::GrpcTransport;
use crate::transport::in_memory::in_memory_transport::InMemoryTransport;
use crate::transport::in_memory::in_memory_transport::Registry;
use crate::transport::tcp::tcp_transport::AddressBook;
use crate::transport::tcp::tcp_transport::TcpTransport;
use crate::transport_kind::TransportKind;

pub trait LateJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport>;
}

pub struct InMemoryJoinMesh {
    registry: Registry,
}

impl InMemoryJoinMesh {
    #[must_use]
    pub fn new(registry: Registry) -> Self {
        Self { registry }
    }
}

impl LateJoinMesh for InMemoryJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(InMemoryTransport::join_mesh(peer_id, self.registry.clone()))
    }
}

pub struct ChannelsJoinMesh {
    registry: ChannelRegistry,
}

impl ChannelsJoinMesh {
    #[must_use]
    pub fn new(registry: ChannelRegistry) -> Self {
        Self { registry }
    }
}

impl LateJoinMesh for ChannelsJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(ChannelsTransport::join_mesh(peer_id, self.registry.clone()))
    }
}

pub struct TcpJoinMesh {
    registry: AddressBook,
}

impl TcpJoinMesh {
    #[must_use]
    pub fn new(registry: AddressBook) -> Self {
        Self { registry }
    }
}

impl LateJoinMesh for TcpJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(TcpTransport::join_mesh(peer_id, self.registry.clone()))
    }
}

pub struct GrpcJoinMesh {
    registry: GrpcAddressBook,
}

impl GrpcJoinMesh {
    #[must_use]
    pub fn new(registry: GrpcAddressBook) -> Self {
        Self { registry }
    }
}

impl LateJoinMesh for GrpcJoinMesh {
    fn connect(&self, peer_id: PeerId) -> Box<dyn Transport> {
        Box::new(GrpcTransport::join_mesh(peer_id, self.registry.clone()))
    }
}

pub struct JoinContext {
    mesh: Box<dyn LateJoinMesh>,
    genesis_peers: Vec<PeerId>,
    node_required: usize,
    delay: Duration,
    writer: Option<SharedWriter>,
}

impl JoinContext {
    #[must_use]
    pub fn new(
        mesh: Box<dyn LateJoinMesh>,
        genesis_peers: Vec<PeerId>,
        node_required: usize,
        delay: Duration,
        writer: Option<SharedWriter>,
    ) -> Self {
        Self {
            mesh,
            genesis_peers,
            node_required,
            delay,
            writer,
        }
    }
}

pub struct ProcessJoinContext {
    spec: ProcessSpec,
    genesis_addrs: Vec<(PeerId, SocketAddr)>,
}

impl ProcessJoinContext {
    #[must_use]
    pub(crate) fn new(spec: ProcessSpec, genesis_addrs: Vec<(PeerId, SocketAddr)>) -> Self {
        Self {
            spec,
            genesis_addrs,
        }
    }
}

pub enum Joining {
    InProcess(JoinContext),
    Process(ProcessJoinContext),
}

pub struct Cluster {
    nodes: Vec<Node>,
    spawn: Spawn,
    poll_delay: Duration,
    started: bool,
    joining: Option<Joining>,
}

impl Cluster {
    #[must_use]
    pub fn new(
        nodes: Vec<Node>,
        spawn: Spawn,
        timer_delay: TimerDelay,
        joining: Option<Joining>,
    ) -> Self {
        Self {
            nodes,
            spawn,
            poll_delay: timer_delay.duration(),
            started: false,
            joining,
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
        let writer = context.writer.clone();

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

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let newcomer_addr = listener.local_addr().unwrap();
        drop(listener);

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
            wait_for_tcp_ready(newcomer_addr, Duration::from_secs(30));
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
    }
}
