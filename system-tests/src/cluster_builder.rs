// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::net::SocketAddr;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use faction::PeerId;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::quorum_policy::QuorumPolicy;

use faction_protocol::protocol::Protocol;
use faction_protocol::timer_trait::Timer;
use faction_protocol::transport_trait::Transport;

use crate::cluster::Cluster;
use crate::faction_node::FactionNode;
use crate::no_op_node_observer::NoOpNodeObserver;
use crate::node::Node;
use crate::node_observer::NodeObserver;
use crate::shared_file_observer::SharedFileObserver;
use crate::shared_file_observer::new_shared_writer;
use crate::spawn::Spawn;
use crate::timer::in_memory::in_memory_timer::InMemoryTimer;
use crate::timer::real::real_timer::RealTimer;
use crate::timer_delay::TimerDelay;
use crate::timer_kind::TimerKind;
use crate::transport::channels::channels_transport::ChannelsTransport;
use crate::transport::grpc::grpc_transport::GrpcTransport;
use crate::transport::in_memory::in_memory_transport::InMemoryTransport;
use crate::transport::tcp::tcp_transport::TcpTransport;
use crate::transport_kind::TransportKind;

pub struct ClusterBuilder {
    node_count: usize,
    required: usize,
    spawn: Spawn,
    transport: TransportKind,
    timer: TimerKind,
    timer_delay: TimerDelay,
    log_path: Option<PathBuf>,
}

impl ClusterBuilder {
    #[must_use]
    pub fn new(node_count: usize, required: usize) -> Self {
        Self {
            node_count,
            required,
            spawn: Spawn::Task,
            transport: TransportKind::InMemory,
            timer: TimerKind::InMemory,
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
    pub fn timer_kind(mut self, timer: TimerKind) -> Self {
        self.timer = timer;
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
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::remove_file(&path);
        self.log_path = Some(path);
        self
    }

    #[must_use]
    pub fn build(self) -> Cluster {
        let peer_ids: Vec<PeerId> = (0..self.node_count as PeerId).collect();

        if matches!(self.spawn, Spawn::Process) {
            return self.build_process(&peer_ids);
        }

        let transports: Vec<Box<dyn Transport>> = match self.transport {
            TransportKind::InMemory => InMemoryTransport::new_mesh(&peer_ids)
                .into_iter()
                .map(|t| Box::new(t) as Box<dyn Transport>)
                .collect(),
            TransportKind::Channels => ChannelsTransport::new_mesh(&peer_ids)
                .into_iter()
                .map(|t| Box::new(t) as Box<dyn Transport>)
                .collect(),
            TransportKind::Tcp => TcpTransport::new_mesh(&peer_ids)
                .into_iter()
                .map(|t| Box::new(t) as Box<dyn Transport>)
                .collect(),
            TransportKind::Grpc => GrpcTransport::new_mesh(&peer_ids)
                .into_iter()
                .map(|t| Box::new(t) as Box<dyn Transport>)
                .collect(),
        };
        let writer = self.log_path.as_ref().map(|p| new_shared_writer(p));
        let delay = self.timer_delay.duration();

        let nodes: Vec<Node> = peer_ids
            .iter()
            .zip(transports)
            .map(|(&id, transport)| {
                let config = Config::new(id, peer_ids.clone(), QuorumPolicy::new(self.required));
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
                let timer: Box<dyn Timer> = match self.timer {
                    TimerKind::InMemory => Box::new(InMemoryTimer::new()),
                    TimerKind::Real => Box::new(RealTimer::with_delay(delay)),
                };
                let faction_node = FactionNode::new(
                    id,
                    peer_ids.clone(),
                    protocol,
                    transport,
                    timer,
                    node_observer,
                    delay,
                );
                match self.spawn {
                    Spawn::Task => Node::task(Arc::new(Mutex::new(faction_node))),
                    Spawn::Thread => Node::spawn_thread(Arc::new(Mutex::new(faction_node))),
                    Spawn::Process => unreachable!(),
                }
            })
            .collect();

        Cluster::new(nodes, self.spawn, self.timer_delay)
    }

    fn build_process(&self, peer_ids: &[PeerId]) -> Cluster {
        let addrs: Vec<SocketAddr> = peer_ids
            .iter()
            .map(|_| {
                let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
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

        let bin = std::env::var("CARGO_BIN_EXE_faction_node")
            .ok()
            .unwrap_or_else(|| {
                let mut path =
                    std::env::current_exe().expect("cannot determine current executable path");
                path.pop();
                path.pop();
                path.push("faction-node");
                path.set_extension(std::env::consts::EXE_EXTENSION);
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
            let timer_arg = match self.timer {
                TimerKind::InMemory => "inmemory",
                TimerKind::Real => "real",
            };

            let mut cmd = Command::new(bin.clone());
            cmd.arg("--peer-id")
                .arg(id.to_string())
                .arg("--peers")
                .arg(&peers_arg)
                .arg("--required")
                .arg(self.required.to_string())
                .arg("--freshness-margin")
                .arg("2")
                .arg("--transport")
                .arg(transport_arg)
                .arg("--timer")
                .arg(timer_arg)
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

        Cluster::new(nodes, self.spawn, self.timer_delay)
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
