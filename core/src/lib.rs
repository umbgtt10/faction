// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![no_std]

extern crate alloc;

pub mod no_op_observer;

pub mod cluster_view;
pub mod command;
pub mod conclusion;
pub mod config;
pub mod faction;
pub mod observer;
pub mod outcome;
pub mod peer_state;
pub mod process_result;
pub mod quorum_policy;
pub mod state;
pub mod states;
pub mod transition;

pub type PeerId = u64;
