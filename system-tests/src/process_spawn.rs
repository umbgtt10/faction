// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

use faction::types::PeerId;

use crate::transport_kind::TransportKind;

pub(crate) struct ProcessSpec {
    pub bin: String,
    pub transport: TransportKind,
    pub node_required: usize,
    pub timer_delay_ms: u128,
    pub freshness_margin: u32,
    pub log_dir: Option<PathBuf>,
}

pub(crate) fn spawn_process_node(
    spec: &ProcessSpec,
    peer_id: PeerId,
    peers: &[PeerId],
    peer_addrs: &[(PeerId, SocketAddr)],
    listen_addr: SocketAddr,
) -> Child {
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
    let transport_arg = match spec.transport {
        TransportKind::Grpc => "grpc",
        TransportKind::Tcp => "tcp",
        _ => panic!("unsupported transport for process node"),
    };

    let mut cmd = Command::new(&spec.bin);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    cmd.arg("--peer-id")
        .arg(peer_id.to_string())
        .arg("--peers")
        .arg(&peers_arg)
        .arg("--required")
        .arg(spec.node_required.to_string())
        .arg("--freshness-margin")
        .arg(spec.freshness_margin.to_string())
        .arg("--transport")
        .arg(transport_arg)
        .arg("--timer-delay")
        .arg(spec.timer_delay_ms.to_string())
        .arg("--listen-addr")
        .arg(listen_addr.to_string())
        .arg("--peer-addrs")
        .arg(&peer_addrs_arg);
    if let Some(ref dir) = spec.log_dir {
        let node_path = dir.join(format!("node-{peer_id}.jsonl"));
        cmd.arg("--log-path")
            .arg(node_path.to_string_lossy().to_string());
    }

    cmd.spawn().expect("failed to spawn faction-node")
}
