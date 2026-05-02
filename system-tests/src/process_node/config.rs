// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::net::SocketAddr;

use faction::PeerId;

use crate::timer_kind::TimerKind;
use crate::transport_kind::TransportKind;

pub struct ProcessNodeConfig {
    pub peer_id: PeerId,
    pub peers: Vec<PeerId>,
    pub required: usize,
    pub freshness_margin: u64,
    pub transport: TransportKind,
    pub timer: TimerKind,
    pub listen_addr: SocketAddr,
    pub peer_addrs: Vec<(PeerId, SocketAddr)>,
}
