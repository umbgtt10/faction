// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;

use crate::join::late_join_mesh::LateJoinMesh;
use crate::node_recipe::NodeRecipe;

pub struct JoinContext {
    pub mesh: Box<dyn LateJoinMesh>,
    pub genesis_peers: Vec<PeerId>,
    pub recipe: NodeRecipe,
}
