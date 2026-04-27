// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![no_std]

extern crate alloc;

pub mod freshness_classification;
pub mod freshness_policy;
pub mod no_op_vibe_observer;

pub mod quorum_policy;
pub mod readiness_exit_mode;
pub mod readiness_lifecycle_state;
pub mod states;
pub mod vibe;
pub mod vibe_config;
pub mod vibe_input;
pub mod vibe_observer;
pub mod vibe_output;
pub mod vibe_snapshot;
pub mod vibe_state;
pub mod vibe_transition;

pub type PeerId = u64;
pub type Freshness = u64;
