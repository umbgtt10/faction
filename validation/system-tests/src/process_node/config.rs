// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;
use std::path::PathBuf;

use crate::transport_kind::TransportKind;

pub struct ProcessNodeConfig {
    pub peer_id: u64,
    pub peers: Vec<u64>,
    pub required: usize,
    pub suspicion_threshold: usize,
    pub bounds: Vec<usize>,
    pub freshness_margin: u64,
    pub probe_stretch: u32,
    pub transport: TransportKind,

    pub listen_addr: SocketAddr,
    pub peer_addrs: Vec<(u64, SocketAddr)>,
    pub log_path: Option<PathBuf>,
    pub timer_delay_ms: u64,
    pub lifetime_ms: u64,
    pub newcomer: bool,
}
