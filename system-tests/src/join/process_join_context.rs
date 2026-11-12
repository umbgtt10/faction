// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;

use faction::types::PeerId;

use crate::infra::network::port_allocator::PortAllocator;
use crate::process_spec::ProcessSpec;

pub struct ProcessJoinContext {
    pub spec: ProcessSpec,
    pub genesis_addrs: Vec<(PeerId, SocketAddr)>,
    pub ports: PortAllocator,
}
