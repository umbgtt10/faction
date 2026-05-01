// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::Arc;
use std::sync::Mutex;

use faction::PeerId;
use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;

use faction_protocol::protocol::Protocol;

use crate::cluster::Cluster;
use crate::faction_node::FactionNode;
use crate::node::Node;
use crate::spawn::Spawn;
use crate::timer::in_memory::in_memory_timer::InMemoryTimer;
use crate::transport::in_memory::in_memory_transport::InMemoryTransport;
use crate::transport_kind::TransportKind;

pub struct ClusterBuilder {
    node_count: usize,
    required: usize,
    spawn: Spawn,
    transport: TransportKind,
}

impl ClusterBuilder {
    #[must_use]
    pub fn new(node_count: usize, required: usize) -> Self {
        Self {
            node_count,
            required,
            spawn: Spawn::Task,
            transport: TransportKind::InMemory,
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
    pub fn build(self) -> Cluster {
        let peer_ids: Vec<PeerId> = (0..self.node_count as PeerId).collect();
        let transports = InMemoryTransport::new_mesh(&peer_ids);

        let nodes: Vec<Node> = peer_ids
            .iter()
            .zip(transports.into_iter())
            .map(|(&id, transport)| {
                let config = Config::new(
                    id,
                    peer_ids.clone(),
                    QuorumPolicy::new(self.required),
                    FreshnessPolicy::new(2),
                );
                let protocol = Protocol::new(
                    Faction::new(config, Box::new(NoOpObserver)),
                    peer_ids.clone(),
                    id,
                );
                let faction_node = FactionNode::new(
                    id,
                    peer_ids.clone(),
                    protocol,
                    Box::new(transport),
                    Box::new(InMemoryTimer::new()),
                );
                Node::task(Arc::new(Mutex::new(faction_node)))
            })
            .collect();

        Cluster::new(nodes)
    }
}
