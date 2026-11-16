// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::constants::DEFAULT_SUSPICION_THRESHOLD;
use crate::probe_stretch::ProbeStretch;
use std::env::consts::EXE_EXTENSION;
use std::env::current_exe;
use std::env::var as env_var;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use faction::types::PeerId;

use faction_protocol::transport::Transport;

use crate::cluster::Cluster;
use crate::constants::DEFAULT_DEADLINE_CYCLES;
use crate::constants::NODE_LIFETIME;
use crate::infra::fault_board::FaultBoard;
use crate::infra::invariants::breaches::Breaches;
use crate::infra::logging::logs::Logs;
use crate::infra::network::port_allocator::PortAllocator;
use crate::infra::network::tcp_readiness::TcpReadiness;
use crate::infra::timer::timer_delays::TimerDelays;
use crate::infra::transport::channels::channels_transport::ChannelsTransport;
use crate::infra::transport::grpc::grpc_transport::GrpcTransport;
use crate::infra::transport::in_memory::in_memory_transport::InMemoryTransport;
use crate::infra::transport::tcp::tcp_transport::TcpTransport;
use crate::join::channels_join_mesh::ChannelsJoinMesh;
use crate::join::grpc_join_mesh::GrpcJoinMesh;
use crate::join::in_memory_join_mesh::InMemoryJoinMesh;
use crate::join::join_context::JoinContext;
use crate::join::joining::Joining;
use crate::join::late_join_mesh::LateJoinMesh;
use crate::join::process_join_context::ProcessJoinContext;
use crate::join::tcp_join_mesh::TcpJoinMesh;
use crate::node::Node;
use crate::node_recipe::NodeRecipe;
use crate::process_spec::ProcessSpec;
use crate::spawn::Spawn;
use crate::time_ports::TimePorts;
use crate::timer_delay::TimerDelay;
use crate::transport_kind::TransportKind;

const PROCESS_READINESS_TIMEOUT: Duration = Duration::from_secs(30);

type TransportsAndJoinMesh = (Vec<Box<dyn Transport>>, Option<Box<dyn LateJoinMesh>>);

pub struct ClusterBuilder {
    node_count: usize,
    node_required: usize,
    time: TimePorts,
    spawn: Spawn,
    transport: TransportKind,
    timer_delay: TimerDelay,
    freshness_margin: u32,
    suspicion_threshold: usize,
    probe_stretch: ProbeStretch,
    log_dir: Option<PathBuf>,
    lifetime: Duration,
    silent: bool,
    faults: FaultBoard,
    breaches: Breaches,
}

impl ClusterBuilder {
    #[must_use]
    pub fn new(node_count: usize, node_required: usize, time: TimePorts) -> Self {
        Self {
            node_count,
            node_required,
            time,
            spawn: Spawn::Task,
            transport: TransportKind::InMemory,
            timer_delay: TimerDelay::Minimal,
            freshness_margin: DEFAULT_DEADLINE_CYCLES,
            suspicion_threshold: DEFAULT_SUSPICION_THRESHOLD,
            probe_stretch: ProbeStretch::Minimal,
            log_dir: None,
            lifetime: NODE_LIFETIME,
            silent: false,
            faults: FaultBoard::default(),
            breaches: Breaches::default(),
        }
    }

    // The board its in-process nodes' invariant observers report into, which
    // the caller keeps to read the first breach.
    #[must_use]
    pub fn breaches(mut self, breaches: Breaches) -> Self {
        self.breaches = breaches;
        self
    }

    #[must_use]
    pub fn build(self) -> Cluster {
        let peer_ids: Vec<PeerId> = (0..self.node_count as PeerId).collect();

        if matches!(self.spawn, Spawn::Process) {
            return self.build_process(&peer_ids);
        }

        let (transports, join_mesh) = self.transports(&peer_ids);
        let delay = self.timer_delay.duration();
        let recipe = NodeRecipe::new(
            self.node_required,
            self.suspicion_threshold,
            TimerDelays {
                delay,
                deadline_delay: delay * self.freshness_margin,
                probe_stretch: self.probe_stretch.factor(),
                lifetime: self.lifetime,
            },
            self.logs(),
            self.time.clone(),
            self.faults.clone(),
            self.breaches.clone(),
        );

        let nodes: Vec<Node> = peer_ids
            .iter()
            .zip(transports)
            .map(|(&id, transport)| recipe.node(self.spawn, id, peer_ids.clone(), transport))
            .collect();

        let joining = join_mesh.map(|mesh| {
            Joining::InProcess(JoinContext {
                mesh,
                genesis_peers: peer_ids.clone(),
                recipe: recipe.clone(),
            })
        });

        Cluster::new(
            nodes,
            peer_ids.clone(),
            self.spawn,
            self.timer_delay,
            joining,
            self.log_dir,
            self.time,
        )
    }

    // The board its in-process nodes read their faults from, which the caller
    // keeps to switch them while the cluster runs; a process node's are its
    // own.
    #[must_use]
    pub fn faults(mut self, board: FaultBoard) -> Self {
        self.faults = board;
        self
    }

    #[must_use]
    pub fn freshness_margin(mut self, cycles: u32) -> Self {
        self.freshness_margin = cycles;
        self
    }

    // How long each node that runs on its own -- a thread's, a process's --
    // runs before it stops, as a backstop against one its cluster never
    // drops: thirty seconds unless set.
    #[must_use]
    pub fn lifetime(mut self, lifetime: Duration) -> Self {
        self.lifetime = lifetime;
        self
    }

    #[must_use]
    pub fn log_dir(mut self, dir: PathBuf) -> Self {
        let _ = remove_dir_all(&dir);
        let _ = create_dir_all(&dir);
        self.log_dir = Some(dir);
        self
    }

    // How many times over its nodes wait the probe period and its timeouts:
    // for nodes that cannot keep up with them at their own -- a task's, which
    // reads one input a step.
    #[must_use]
    pub fn probe_stretch(mut self, stretch: ProbeStretch) -> Self {
        self.probe_stretch = stretch;
        self
    }

    // A cluster whose nodes announce none of the faults they inject, nor log
    // anything, unless given a log folder.
    #[must_use]
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
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

    // How many probe periods running a member must miss before its nodes
    // suspect it.
    #[must_use]
    pub fn suspicion_threshold(mut self, threshold: usize) -> Self {
        self.suspicion_threshold = threshold;
        self
    }

    #[must_use]
    pub fn timer_delay(mut self, timer_delay: TimerDelay) -> Self {
        self.timer_delay = timer_delay;
        self
    }

    #[must_use]
    pub fn transport(mut self, transport: TransportKind) -> Self {
        self.transport = transport;
        self
    }

    fn build_process(self, peer_ids: &[PeerId]) -> Cluster {
        // Under slotgate each parallel slot gets a disjoint PORT_RANGE, so a port
        // derived per node from its base avoids the cross-test port-reuse race of
        // bind(":0")-drop-rebind; outside slotgate the ports are ephemeral.
        let ports = PortAllocator::new(
            env_var("PORT_RANGE_BASE")
                .ok()
                .and_then(|value| value.parse::<u16>().ok()),
        );
        let addrs: Vec<SocketAddr> = (0..peer_ids.len()).map(|index| ports.addr(index)).collect();
        let peer_addrs: Vec<(PeerId, SocketAddr)> = peer_ids
            .iter()
            .copied()
            .zip(addrs.iter().copied())
            .collect();

        let spec = ProcessSpec::new(
            Self::node_binary(),
            self.transport,
            self.node_required,
            self.suspicion_threshold,
            self.timer_delay.duration(),
            self.freshness_margin,
            self.probe_stretch.factor(),
            self.log_dir.clone(),
            self.lifetime,
            TcpReadiness::new(self.time.clone(), PROCESS_READINESS_TIMEOUT),
        );

        let nodes = peer_ids
            .iter()
            .zip(&addrs)
            .map(|(&id, &addr)| spec.launch(id, peer_ids, &peer_addrs, addr))
            .collect();

        let joining = Joining::Process(ProcessJoinContext {
            spec,
            genesis_addrs: peer_addrs,
            ports,
        });
        Cluster::new(
            nodes,
            peer_ids.to_vec(),
            self.spawn,
            self.timer_delay,
            Some(joining),
            self.log_dir,
            self.time,
        )
    }

    // Where its in-process nodes log: their folder, if given one; else
    // nowhere, if silent; else their faults on stderr.
    fn logs(&self) -> Logs {
        match (&self.log_dir, self.silent) {
            (Some(dir), _) => Logs::Files(dir.clone()),
            (None, true) => Logs::Silent,
            (None, false) => Logs::Stderr,
        }
    }

    // The process node's binary: beside the running one -- a binary's, as
    // the chaos binary is -- or a folder above it -- a test binary's, which
    // runs from deps/.
    fn node_binary() -> String {
        env_var("CARGO_BIN_EXE_faction_node")
            .ok()
            .unwrap_or_else(|| {
                let exe = current_exe().expect("cannot determine current executable path");
                let beside = exe
                    .with_file_name("faction-node")
                    .with_extension(EXE_EXTENSION);
                let path = if beside.exists() {
                    beside
                } else {
                    let mut above = exe;
                    above.pop();
                    above.pop();
                    above.push("faction-node");
                    above.set_extension(EXE_EXTENSION);
                    above
                };
                path.to_string_lossy().to_string()
            })
    }

    fn transports(&self, peer_ids: &[PeerId]) -> TransportsAndJoinMesh {
        match self.transport {
            TransportKind::InMemory => {
                let mesh = InMemoryTransport::new_mesh(peer_ids);
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
                let mesh = ChannelsTransport::new_mesh(peer_ids);
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
                let mesh = TcpTransport::new_mesh(peer_ids, self.time.pause.clone());
                let join_mesh = mesh.first().map(|t| {
                    Box::new(TcpJoinMesh::new(t.registry(), self.time.pause.clone()))
                        as Box<dyn LateJoinMesh>
                });
                let transports = mesh
                    .into_iter()
                    .map(|t| Box::new(t) as Box<dyn Transport>)
                    .collect();
                (transports, join_mesh)
            }
            TransportKind::Grpc => {
                let mesh = GrpcTransport::new_mesh(peer_ids);
                let join_mesh = mesh
                    .first()
                    .map(|t| Box::new(GrpcJoinMesh::new(t.registry())) as Box<dyn LateJoinMesh>);
                let transports = mesh
                    .into_iter()
                    .map(|t| Box::new(t) as Box<dyn Transport>)
                    .collect();
                (transports, join_mesh)
            }
        }
    }
}
