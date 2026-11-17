// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::cluster::Cluster;
use faction_system_tests::cluster_builder::ClusterBuilder;

use crate::model::scenario::Scenario;

// What a chaos run is played on: the builder of a scenario's cluster -- its
// nodes' spawn, their transport and their time -- and what a round does to
// the cluster besides the layer above's acts.
pub trait ChaosEngine {
    fn builder(&self, scenario: &Scenario) -> ClusterBuilder;

    fn round(&self, cluster: &Cluster);
}
