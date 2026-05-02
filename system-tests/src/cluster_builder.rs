// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use faction::PeerId;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
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
            log_path: None,
        }
    }

    #[must_use]
    pub fn spawn(mut self, spawn: Spawn) -> Self {
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

        let nodes: Vec<Node> = peer_ids
            .iter()
            .zip(transports)
            .map(|(&id, transport)| {
                let config = Config::new(
                    id,
                    peer_ids.clone(),
                    QuorumPolicy::new(self.required),
                    FreshnessPolicy::new(2),
                );
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
                    TimerKind::Real => Box::new(RealTimer::new()),
                };
                let faction_node = FactionNode::new(
                    id,
                    peer_ids.clone(),
                    protocol,
                    transport,
                    timer,
                    node_observer,
                );
                match self.spawn {
                    Spawn::Task => Node::task(Arc::new(Mutex::new(faction_node))),
                    Spawn::Thread => Node::spawn_thread(Arc::new(Mutex::new(faction_node))),
                    _ => unimplemented!(),
                }
            })
            .collect();

        Cluster::new(nodes)
    }
}
