// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![no_std]

extern crate alloc;

pub mod cluster_readiness;
pub mod cluster_readiness_config;
pub mod cluster_readiness_input;
pub mod cluster_readiness_observer;
pub mod cluster_readiness_output;
pub mod cluster_readiness_snapshot;
pub mod cluster_readiness_transition;
pub mod freshness_classification;
pub mod freshness_policy;
pub mod no_op_cluster_readiness_observer;
pub mod output_batch;
pub mod quorum_policy;
pub mod readiness_exit_mode;
pub mod readiness_lifecycle_state;

pub type PeerId = u64;
pub type Freshness = u64;
