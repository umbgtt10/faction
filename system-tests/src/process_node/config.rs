// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
