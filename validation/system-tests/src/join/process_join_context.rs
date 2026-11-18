// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;

use crate::infra::network::port_allocator::PortAllocator;
use crate::process_spec::ProcessSpec;

pub struct ProcessJoinContext {
    pub spec: ProcessSpec,
    pub genesis_addrs: Vec<(u64, SocketAddr)>,
    pub ports: PortAllocator,
}
