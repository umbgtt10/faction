// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

//! # faction-spec-model
//!
//! **An independent model of faction's specification.**
//!
//! The specification in `spec/` is normative, and the `faction` crate is one
//! implementation of it. This is a second one, written from the specification's
//! prose and decision table alone, and kept apart from the crate: it depends on
//! nothing of `faction`, and the two meet only in the specification's own
//! encoding, in the ladder. Its vectors are checked against it before the crate
//! runs them, and the crate is checked against it over command sequences no
//! vector holds.
//!
//! Its shape is the specification's rule, not the crate's states: a command is
//! answered by the first row of the decision table that applies, in the table's
//! order. The model reads the rows, and owns only what the prose gives each one
//! -- the situation in which it applies and the change it makes -- the peers its
//! outcomes name, and the view.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod addition;
pub mod answer;
pub mod command;
pub mod configuration;
pub mod liveness;
pub mod model;
pub mod node_state;
pub mod rules;
pub mod spec_reader;
pub mod table;
pub mod view;
pub mod vocabulary;
