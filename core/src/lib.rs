// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

//! # faction
//!
//! **A deterministic, `no_std` Mealy state machine for cluster bootstrapping.**
//!
//! Every distributed system has a moment where it stops being a pile of processes
//! and starts being a cluster. That moment is *bootstrapping* — and it's usually
//! the least-tested, most-fragile code in the entire stack.
//!
//! `faction` replaces ad-hoc coordination with a formally specified state machine
//! that answers one question: **is the cluster ready to proceed?**
//!
//! The terminal answer is `Bootstrapped`. A blown deadline is recorded as
//! `TimedOut` but is never terminal — the node keeps trying.
//!
//! ## The pitch
//!
//! You bring the network, the transport, and the definition of "ready."
//! `faction` brings the state transitions — every single one of them tested,
//! observable, and replayable from an input log.
//!
//! * **Protocol-agnostic** — no opinion on what a peer *is* or how messages move
//! * **Deterministic** — same inputs → same outputs, always
//! * **Exhaustively tested** — every `(state, command)` pair covered by an explicit matrix
//! * **Zero unsafe** — `#![forbid(unsafe_code)]`
//! * **`no_std + alloc`** — runs on bare metal, WASM, embedded, and cloud
//!
//! Design rationale is recorded as Architecture Decision Records in
//! [`docs/ADRs/`](https://github.com/umbgtt10/faction/tree/main/docs/ADRs).
//!
//! ## Example
//!
//! ```rust
//! use faction::command::Command;
//! use faction::config::Config;
//! use faction::faction::Faction;
//! use faction::no_op_observer::NoOpObserver;
//! use faction::process_result::ProcessResult;
//! use faction::quorum_policy::QuorumPolicy;
//!
//! extern crate alloc;
//!
//! // A 5-node cluster. We need 4 to agree before proceeding.
//! let config = Config::new(
//!     0,                                 // our peer id
//!     alloc::vec![0, 1, 2, 3, 4],        // all peers
//!     QuorumPolicy::new(4),              // quorum threshold
//! );
//!
//! let mut machine = Faction::new(config, Box::new(NoOpObserver));
//!
//! // Phase 1 — feed participation signals as they arrive from the wire.
//! assert!(matches!(
//!     machine.process(Command::ParticipationObserved { peer_id: 1 }),
//!     ProcessResult::Accepted { .. }
//! ));
//! machine.process(Command::ParticipationObserved { peer_id: 2 });
//!
//! // A duplicate is accepted but marked ignored — and, like every result,
//! // it reports the set of commands admissible next.
//! if let ProcessResult::Accepted { admissible, .. } =
//!     machine.process(Command::ParticipationObserved { peer_id: 1 })
//! {
//!     assert!(admissible.contains(&Command::LocalParticipationCompleted));
//! }
//!
//! // Probe at any time — read-only, zero side effects.
//! if let ProcessResult::Probed { cluster_view, .. } =
//!     machine.process(Command::Probe)
//! {
//!     assert_eq!(cluster_view.pinging_peers(), &[1, 2]);
//! }
//!
//! // Phase 2 — local participation done, now collecting readiness.
//! machine.process(Command::LocalParticipationCompleted);
//! machine.process(Command::ReadyObserved { peer_id: 1 });
//! machine.process(Command::ReadyObserved { peer_id: 2 });
//!
//! // Self plus peers 1, 2, 3 = 4 confirmations → quorum → Bootstrapped.
//! let result = machine.process(Command::ReadyObserved { peer_id: 3 });
//! if let ProcessResult::Accepted { cluster_view, .. } = result {
//!     assert!(cluster_view.is_concluded());
//!     // The cluster is live. Hand off to the application.
//! }
//! ```
//!
//! ## State machine
//!
//! ```text
//! Initial → Pinging → Collecting → Bootstrapped
//! ```
//!
//! | State | Carries |
//! |---|---|
//! | `Initial` | Nothing — unit struct |
//! | `Pinging` | Active pinging and collecting peer sets |
//! | `Collecting` | Collecting and pinged peer sets |
//! | `Bootstrapped` | Terminal — quorum reached |
//!
//! `Bootstrapped` is the only terminal state. A missed deadline is recorded as
//! a fact — surfaced as `PeerState::TimedOut` — without leaving the current
//! state, so the node stays receptive and can still converge. And a
//! bootstrapped node answers a still-pinging peer with its readiness, so a
//! concluded node is never a silent sink.
//!
//! ## Observer
//!
//! Every transition fires a callback through the [`Observer`] trait. Wire it
//! to telemetry, an audit log, or a test assertion. The machine doesn't care.
//! [`NoOpObserver`] is provided for the common "just drive the machine" case.
//!
//! ## Further reading
//!
//! * [README](https://crates.io/crates/faction) — project overview and design principles
//! * [Architecture](https://github.com/umbgtt10/faction/blob/main/docs/ARCHITECTURE.md)
//! * [Transition matrix tests](https://github.com/umbgtt10/faction/blob/main/core/tests/transition_matrix/) — exhaustive `(state × command)` coverage

#![no_std]
#![forbid(unsafe_code)]

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
pub mod types;

#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
pub struct ReadmeDoctests;
