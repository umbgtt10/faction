// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::env::current_exe;
use std::env::var as env_var;
use std::fs::{create_dir_all, remove_dir_all};
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::PathBuf;
use std::rc::Rc;

use std::env::consts::EXE_EXTENSION;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::quorum_policy::QuorumPolicy;
use faction::types::PeerId;

use faction_protocol::protocol::Protocol;
use faction_protocol::timer_trait::Timer;
use faction_protocol::transport_trait::Transport;

use crate::cluster::ChannelsJoinMesh;
use crate::cluster::Cluster;
use crate::cluster::GrpcJoinMesh;
use crate::cluster::InMemoryJoinMesh;
use crate::cluster::JoinContext;
use crate::cluster::Joining;
use crate::cluster::LateJoinMesh;
use crate::cluster::ProcessJoinContext;
use crate::cluster::TcpJoinMesh;
use crate::faction_node::FactionNode;
use crate::no_op_node_observer::NoOpNodeObserver;
use crate::node::Node;
use crate::node_observer::NodeObserver;
use crate::process_spawn::ProcessSpec;
use crate::process_spawn::spawn_process_node;
use crate::shared_file_observer::SharedFileObserver;
use crate::shared_file_observer::SharedWriter;
use crate::shared_file_observer::new_shared_writer;
use crate::spawn::Spawn;
use crate::timer::real::real_timer::DEFAULT_DEADLINE_CYCLES;
use crate::timer::real::real_timer::RealTimer;
use crate::timer_delay::TimerDelay;

use crate::transport::channels::channels_transport::ChannelsTransport;
use crate::transport::grpc::grpc_transport::GrpcTransport;
use crate::transport::in_memory::in_memory_transport::InMemoryTransport;
use crate::transport::tcp::tcp_transport::TcpTransport;
use crate::transport_kind::TransportKind;

type TransportsAndJoinMesh = (Vec<Box<dyn Transport>>, Option<Box<dyn LateJoinMesh>>);

pub struct ClusterBuilder {
    node_count: usize,
    node_required: usize,
    spawn: Spawn,
    transport: TransportKind,
    timer_delay: TimerDelay,
    freshness_margin: u32,
    log_dir: Option<PathBuf>,
}

impl ClusterBuilder {
    #[must_use]
    pub fn new(node_count: usize, node_required: usize) -> Self {
        Self {
            node_count,
            node_required,
            spawn: Spawn::Task,
            transport: TransportKind::InMemory,
            timer_delay: TimerDelay::Minimal,
            freshness_margin: DEFAULT_DEADLINE_CYCLES,
            log_dir: None,
        }
    }

    #[must_use]
    pub fn spawn(mut self, spawn: Spawn) -> Self {
        self.timer_delay = match spawn {
            Spawn::Task => TimerDelay::Minimal,
            Spawn::Thread => TimerDelay::Moderate,
            Spawn::Process => TimerDelay::Generous,
        };
        self.spawn = spawn;
        self
    }

    #[must_use]
    pub fn transport(mut self, transport: TransportKind) -> Self {
        self.transport = transport;
        self
    }

    #[must_use]
    pub fn timer_delay(mut self, timer_delay: TimerDelay) -> Self {
        self.timer_delay = timer_delay;
        self
    }

    #[must_use]
    pub fn freshness_margin(mut self, cycles: u32) -> Self {
        self.freshness_margin = cycles;
        self
    }

    #[must_use]
    pub fn log_dir(mut self, dir: PathBuf) -> Self {
        let _ = remove_dir_all(&dir);
        let _ = create_dir_all(&dir);
        self.log_dir = Some(dir);
        self
    }

    #[must_use]
    pub fn build(self) -> Cluster {
        let peer_ids: Vec<PeerId> = (0..self.node_count as PeerId).collect();

        if matches!(self.spawn, Spawn::Process) {
            return self.build_process(&peer_ids);
        }

        let (transports, join_mesh): TransportsAndJoinMesh = match self.transport {
            TransportKind::InMemory => {
                let mesh = InMemoryTransport::new_mesh(&peer_ids);
                let join_mesh = mesh.first().map(|t| {
                    Box::new(InMemoryJoinMesh::new(t.registry())) as Box<dyn LateJoinMesh>
                });
                let transports = mesh
                    .into_iter()
                    .map(|t| Box::new(t) as Box<dyn Transport>)
                    .collect();
                (transports, join_mesh)
            }
            TransportKind::Channels => {
                let mesh = ChannelsTransport::new_mesh(&peer_ids);
                let join_mesh = mesh.first().map(|t| {
                    Box::new(ChannelsJoinMesh::new(t.registry())) as Box<dyn LateJoinMesh>
                });
                let transports = mesh
                    .into_iter()
                    .map(|t| Box::new(t) as Box<dyn Transport>)
                    .collect();
                (transports, join_mesh)
            }
            TransportKind::Tcp => {
                let mesh = TcpTransport::new_mesh(&peer_ids);
                let join_mesh = mesh
                    .first()
                    .map(|t| Box::new(TcpJoinMesh::new(t.registry())) as Box<dyn LateJoinMesh>);
                let transports = mesh
                    .into_iter()
                    .map(|t| Box::new(t) as Box<dyn Transport>)
                    .collect();
                (transports, join_mesh)
            }
            TransportKind::Grpc => {
                let mesh = GrpcTransport::new_mesh(&peer_ids);
                let join_mesh = mesh
                    .first()
                    .map(|t| Box::new(GrpcJoinMesh::new(t.registry())) as Box<dyn LateJoinMesh>);
                let transports = mesh
                    .into_iter()
                    .map(|t| Box::new(t) as Box<dyn Transport>)
                    .collect();
                (transports, join_mesh)
            }
        };
        let log_dir = self.log_dir.clone();
        let delay = self.timer_delay.duration();
        let deadline_delay = delay * self.freshness_margin;
        let node_required = self.node_required;
        let spawn = self.spawn;

        let nodes: Vec<Node> = peer_ids
            .iter()
            .zip(transports)
            .map(|(&id, transport)| {
                if matches!(spawn, Spawn::Thread) {
                    let thread_writer = node_writer(&log_dir, id);
                    let thread_peer_ids = peer_ids.clone();
                    let timer: Box<dyn Timer> =
                        Box::new(RealTimer::with_delays(delay, deadline_delay));
                    return Node::spawn_thread(move || {
                        let faction_observer: Box<dyn Observer> = match &thread_writer {
                            Some(w) => Box::new(SharedFileObserver::new(w.clone(), id)),
                            None => Box::new(NoOpObserver),
                        };
                        let node_observer: Box<dyn NodeObserver> = match &thread_writer {
                            Some(w) => Box::new(SharedFileObserver::new(w.clone(), id)),
                            None => Box::new(NoOpNodeObserver),
                        };
                        let config = Config::new(
                            id,
                            thread_peer_ids.clone(),
                            QuorumPolicy::new(node_required),
                        );
                        let protocol = Protocol::new(
                            Faction::new(config, faction_observer),
                            thread_peer_ids.clone(),
                            id,
                        );
                        FactionNode::new(
                            id,
                            thread_peer_ids,
                            protocol,
                            transport,
                            timer,
                            node_observer,
                            delay,
                        )
                    });
                }

                let config = Config::new(id, peer_ids.clone(), QuorumPolicy::new(node_required));
                let task_writer = node_writer(&log_dir, id);
                let faction_observer: Box<dyn Observer> = match &task_writer {
                    Some(w) => Box::new(SharedFileObserver::new(w.clone(), id)),
                    None => Box::new(NoOpObserver),
                };
                let node_observer: Box<dyn NodeObserver> = match &task_writer {
                    Some(w) => Box::new(SharedFileObserver::new(w.clone(), id)),
                    None => Box::new(NoOpNodeObserver),
                };
                let protocol =
                    Protocol::new(Faction::new(config, faction_observer), peer_ids.clone(), id);
                let timer: Box<dyn Timer> = Box::new(RealTimer::with_delays(delay, deadline_delay));
                let faction_node = FactionNode::new(
                    id,
                    peer_ids.clone(),
                    protocol,
                    transport,
                    timer,
                    node_observer,
                    delay,
                );
                Node::task(Rc::new(RefCell::new(faction_node)))
            })
            .collect();

        let joining = join_mesh.map(|mesh| {
            Joining::InProcess(JoinContext::new(
                mesh,
                peer_ids.clone(),
                node_required,
                delay,
                log_dir.clone(),
            ))
        });

        Cluster::new(nodes, spawn, self.timer_delay, joining, log_dir)
    }

    fn build_process(&self, peer_ids: &[PeerId]) -> Cluster {
        let addrs: Vec<SocketAddr> = peer_ids
            .iter()
            .enumerate()
            .map(|(index, _)| Self::allocate_port_addr(index))
            .collect();

        let peer_addrs: Vec<(PeerId, SocketAddr)> = peer_ids
            .iter()
            .copied()
            .zip(addrs.iter().copied())
            .collect();

        let bin = env_var("CARGO_BIN_EXE_faction_node")
            .ok()
            .unwrap_or_else(|| {
                let mut path = current_exe().expect("cannot determine current executable path");
                path.pop();
                path.pop();
                path.push("faction-node");
                path.set_extension(EXE_EXTENSION);
                path.to_string_lossy().to_string()
            });

        let spec = ProcessSpec {
            bin,
            transport: self.transport,
            node_required: self.node_required,
            timer_delay_ms: self.timer_delay.duration().as_millis(),
            freshness_margin: self.freshness_margin,
            log_dir: self.log_dir.clone(),
        };

        let mut nodes = Vec::new();
        for (i, &id) in peer_ids.iter().enumerate() {
            let child = spawn_process_node(&spec, id, peer_ids, &peer_addrs, addrs[i]);
            if self.transport == TransportKind::Tcp {
                Self::wait_for_tcp_ready(addrs[i], Duration::from_secs(30));
            }
            nodes.push(Node::process(child));
        }

        let joining = Joining::Process(ProcessJoinContext::new(spec, peer_addrs));
        Cluster::new(
            nodes,
            self.spawn,
            self.timer_delay,
            Some(joining),
            self.log_dir.clone(),
        )
    }

    pub(crate) fn wait_for_tcp_ready(addr: SocketAddr, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if TcpStream::connect(addr).is_ok() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("Process TCP listener not ready within timeout");
    }

    // Under slotgate each parallel slot gets a disjoint PORT_RANGE, so deriving a
    // distinct port per node from that base avoids the cross-test port-reuse race of
    // bind(":0")-drop-rebind. Falls back to an ephemeral port when run outside slotgate.
    pub(crate) fn allocate_port_addr(index: usize) -> SocketAddr {
        let range_base = env_var("PORT_RANGE_BASE")
            .ok()
            .and_then(|value| value.parse::<u16>().ok());
        if let Some(base) = range_base {
            let port = base.saturating_add(u16::try_from(index).unwrap_or(0));
            return SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        addr
    }
}

pub(crate) fn node_writer(log_dir: &Option<PathBuf>, id: PeerId) -> Option<SharedWriter> {
    log_dir
        .as_ref()
        .map(|dir| new_shared_writer(&dir.join(format!("node-{id}.jsonl"))))
}
