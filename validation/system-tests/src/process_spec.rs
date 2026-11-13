// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use faction::types::PeerId;

use crate::infra::network::tcp_readiness::TcpReadiness;
use crate::node::Node;
use crate::transport_kind::TransportKind;

pub struct ProcessSpec {
    bin: String,
    transport: TransportKind,
    node_required: usize,
    suspicion_threshold: usize,
    timer_delay: Duration,
    freshness_margin: u32,
    log_dir: Option<PathBuf>,
    readiness: TcpReadiness,
}

impl ProcessSpec {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bin: String,
        transport: TransportKind,
        node_required: usize,
        suspicion_threshold: usize,
        timer_delay: Duration,
        freshness_margin: u32,
        log_dir: Option<PathBuf>,
        readiness: TcpReadiness,
    ) -> Self {
        Self {
            bin,
            transport,
            node_required,
            suspicion_threshold,
            timer_delay,
            freshness_margin,
            log_dir,
            readiness,
        }
    }

    #[must_use]
    pub fn launch(
        &self,
        peer_id: PeerId,
        peers: &[PeerId],
        peer_addrs: &[(PeerId, SocketAddr)],
        listen_addr: SocketAddr,
    ) -> Node {
        self.spawn(
            self.command(peer_id, peers, peer_addrs, listen_addr),
            listen_addr,
        )
    }

    // A newcomer, waiting for its membership: it knows its peers' addresses,
    // not who its members are.
    #[must_use]
    pub fn launch_newcomer(
        &self,
        peer_id: PeerId,
        peer_addrs: &[(PeerId, SocketAddr)],
        listen_addr: SocketAddr,
    ) -> Node {
        let mut command = self.command(peer_id, &[peer_id], peer_addrs, listen_addr);
        command.arg("--newcomer");
        self.spawn(command, listen_addr)
    }

    fn command(
        &self,
        peer_id: PeerId,
        peers: &[PeerId],
        peer_addrs: &[(PeerId, SocketAddr)],
        listen_addr: SocketAddr,
    ) -> Command {
        let peers_arg = peers
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let peer_addrs_arg = peer_addrs
            .iter()
            .map(|(pid, a)| format!("{pid}={a}"))
            .collect::<Vec<_>>()
            .join(",");
        let transport_arg = match self.transport {
            TransportKind::Grpc => "grpc",
            TransportKind::Tcp => "tcp",
            _ => panic!("unsupported transport for process node"),
        };

        let mut command = Command::new(&self.bin);
        command.stdin(Stdio::piped()).stdout(Stdio::piped());
        command
            .arg("--peer-id")
            .arg(peer_id.to_string())
            .arg("--peers")
            .arg(&peers_arg)
            .arg("--required")
            .arg(self.node_required.to_string())
            .arg("--suspicion-threshold")
            .arg(self.suspicion_threshold.to_string())
            .arg("--freshness-margin")
            .arg(self.freshness_margin.to_string())
            .arg("--transport")
            .arg(transport_arg)
            .arg("--timer-delay")
            .arg(self.timer_delay.as_millis().to_string())
            .arg("--listen-addr")
            .arg(listen_addr.to_string())
            .arg("--peer-addrs")
            .arg(&peer_addrs_arg);
        if let Some(ref dir) = self.log_dir {
            let node_path = dir.join(format!("node-{peer_id}.jsonl"));
            command
                .arg("--log-path")
                .arg(node_path.to_string_lossy().to_string());
        }
        command
    }

    fn spawn(&self, mut command: Command, listen_addr: SocketAddr) -> Node {
        let child = command.spawn().expect("failed to spawn faction-node");
        if self.transport == TransportKind::Tcp {
            self.readiness.wait(listen_addr);
        }
        Node::process(child)
    }
}
