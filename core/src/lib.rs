// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![no_std]

extern crate alloc;

pub mod freshness_classification;
pub mod freshness_policy;
pub mod no_op_machine_observer;

pub mod machine;
pub mod machine_config;
pub mod machine_input;
pub mod machine_observer;
pub mod machine_output;
pub mod machine_snapshot;
pub mod machine_state;
pub mod machine_transition;
pub mod quorum_policy;
pub mod readiness_exit_mode;
pub mod readiness_lifecycle_state;
pub mod states;

pub type PeerId = u64;
pub type Freshness = u64;
