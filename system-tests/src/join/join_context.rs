// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::time::Duration;

use faction::types::PeerId;

use super::late_join_mesh::LateJoinMesh;

pub struct JoinContext {
    pub mesh: Box<dyn LateJoinMesh>,
    pub genesis_peers: Vec<PeerId>,
    pub node_required: usize,
    pub delay: Duration,
    pub log_dir: Option<PathBuf>,
}

impl JoinContext {
    #[must_use]
    pub fn new(
        mesh: Box<dyn LateJoinMesh>,
        genesis_peers: Vec<PeerId>,
        node_required: usize,
        delay: Duration,
        log_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            mesh,
            genesis_peers,
            node_required,
            delay,
            log_dir,
        }
    }
}
