// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;

use faction::types::PeerId;

use crate::process_spawn::ProcessSpec;

pub struct ProcessJoinContext {
    pub spec: ProcessSpec,
    pub genesis_addrs: Vec<(PeerId, SocketAddr)>,
}

impl ProcessJoinContext {
    #[must_use]
    pub fn new(spec: ProcessSpec, genesis_addrs: Vec<(PeerId, SocketAddr)>) -> Self {
        Self {
            spec,
            genesis_addrs,
        }
    }
}
