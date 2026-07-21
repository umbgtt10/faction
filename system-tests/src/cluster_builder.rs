// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::env::current_exe;
use std::env::var as env_var;
use std::fs::{create_dir_all, remove_file};
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
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
use crate::cluster::LateJoinMesh;
use crate::cluster::TcpJoinMesh;
use crate::faction_node::FactionNode;
use crate::no_op_node_observer::NoOpNodeObserver;
use crate::node::Node;
use crate::node_observer::NodeObserver;
use crate::shared_file_observer::SharedFileObserver;
use crate::shared_file_observer::new_shared_writer;
use crate::spawn::Spawn;
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
    log_path: Option<PathBuf>,
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
            log_path: None,
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
    pub fn log_path(mut self, path: PathBuf) -> Self {
        if let Some(parent) = path.parent() {
            let _ = create_dir_all(parent);
        }
        let _ = remove_file(&path);
        self.log_path = Some(path);
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
        let writer = self.log_path.as_ref().map(|p| new_shared_writer(p));
        let delay = self.timer_delay.duration();
        let node_required = self.node_required;
        let spawn = self.spawn;

        let nodes: Vec<Node> = peer_ids
            .iter()
            .zip(transports)
            .map(|(&id, transport)| {
                if matches!(spawn, Spawn::Thread) {
                    let thread_writer = writer.clone();
                    let thread_peer_ids = peer_ids.clone();
                    let timer: Box<dyn Timer> = Box::new(RealTimer::with_delay(delay));
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
                let faction_observer: Box<dyn Observer> = match &writer {
                    Some(w) => Box::new(SharedFileObserver::new(w.clone(), id)),
                    None => Box::new(NoOpObserver),
                };
                let node_observer: Box<dyn NodeObserver> = match &writer {
                    Some(w) => Box::new(SharedFileObserver::new(w.clone(), id)),
                    None => Box::new(NoOpNodeObserver),
                };
                let protocol =
                    Protocol::new(Faction::new(config, faction_observer), peer_ids.clone(), id);
                let timer: Box<dyn Timer> = Box::new(RealTimer::with_delay(delay));
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

        let join_context = match (spawn, self.transport) {
            (
                Spawn::Task,
                TransportKind::InMemory
                | TransportKind::Channels
                | TransportKind::Tcp
                | TransportKind::Grpc,
            ) => join_mesh.map(|mesh| {
                JoinContext::new(mesh, peer_ids.clone(), node_required, delay, writer.clone())
            }),
            _ => None,
        };

        Cluster::new(nodes, spawn, self.timer_delay, join_context)
    }

    fn build_process(&self, peer_ids: &[PeerId]) -> Cluster {
        let addrs: Vec<SocketAddr> = peer_ids
            .iter()
            .map(|_| {
                let l = TcpListener::bind("127.0.0.1:0").unwrap();
                let a = l.local_addr().unwrap();
                drop(l);
                a
            })
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

        let timer_delay_arg = format!("{}", self.timer_delay.duration().as_millis());

        let mut nodes = Vec::new();
        for (i, &id) in peer_ids.iter().enumerate() {
            let peer_addrs_arg = peer_addrs
                .iter()
                .map(|(pid, a)| format!("{pid}={a}"))
                .collect::<Vec<_>>()
                .join(",");
            let peers_arg = peer_ids
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let transport_arg = match self.transport {
                TransportKind::Grpc => "grpc",
                TransportKind::Tcp => "tcp",
                _ => panic!("unsupported transport for process node"),
            };
            let mut cmd = Command::new(bin.clone());
            cmd.arg("--peer-id")
                .arg(id.to_string())
                .arg("--peers")
                .arg(&peers_arg)
                .arg("--required")
                .arg(self.node_required.to_string())
                .arg("--freshness-margin")
                .arg("2")
                .arg("--transport")
                .arg(transport_arg)
                .arg("--timer-delay")
                .arg(&timer_delay_arg)
                .arg("--listen-addr")
                .arg(addrs[i].to_string())
                .arg("--peer-addrs")
                .arg(&peer_addrs_arg);

            if let Some(ref log_path) = self.log_path {
                cmd.arg("--log-path")
                    .arg(log_path.to_string_lossy().to_string());
            }

            let child = cmd.spawn().expect("failed to spawn faction-node");

            if self.transport == TransportKind::Tcp {
                wait_for_tcp_ready(addrs[i], Duration::from_secs(30));
            }

            nodes.push(Node::process(child));
        }

        Cluster::new(nodes, self.spawn, self.timer_delay, None)
    }
}

fn wait_for_tcp_ready(addr: SocketAddr, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("Process TCP listener not ready within timeout");
}
