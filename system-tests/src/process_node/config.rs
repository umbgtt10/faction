// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;
use std::path::PathBuf;

use faction::types::PeerId;

use crate::transport_kind::TransportKind;

pub struct ProcessNodeConfig {
    pub peer_id: PeerId,
    pub peers: Vec<PeerId>,
    pub required: usize,
    pub freshness_margin: u64,
    pub transport: TransportKind,

    pub listen_addr: SocketAddr,
    pub peer_addrs: Vec<(PeerId, SocketAddr)>,
    pub log_path: Option<PathBuf>,
    pub timer_delay_ms: u64,
}
