// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

//! The testing ladder's memory tier: a counting allocator, measured where the
//! allocator sees every `Layout` rather than where the machine believes it
//! is, and what it measures -- a conclusion, probe periods and membership
//! changes, each against the README's claim that the machine is slim by
//! construction. Not published: it exists for its tests.

pub mod allocation_counter;
pub mod conclusion_sequence;
pub mod counting_allocator;
pub mod liveness_sequence;
pub mod memory_profile;
pub mod reconfiguration_sequence;
