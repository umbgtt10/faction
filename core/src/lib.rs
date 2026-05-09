// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
//! The answer is always `Bootstrapped` or `TimedOut`. No ambiguity.
//!
//! ## The pitch
//!
//! You bring the network, the transport, and the definition of "ready."
//! `faction` brings the state transitions — every single one of them tested,
//! observable, and replayable from an input log.
//!
//! * **Protocol-agnostic** — no opinion on what a peer *is* or how messages move
//! * **Deterministic** — same inputs → same outputs, always
//! * **Exhaustively tested** — 264 tests cover every `(state, command)` pair
//! * **Zero unsafe** — `#![deny(unsafe_code)]`
//! * **`no_std + alloc`** — runs on bare metal, WASM, embedded, and cloud
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
//! assert!(matches!(
//!     machine.process(Command::ParticipationObserved { peer_id: 2 }),
//!     ProcessResult::Accepted { .. }
//! ));
//!
//! // Duplicate signal? The machine rejects it, tells you why, and tells you
//! // what IS valid right now.
//! let result = machine.process(Command::ParticipationObserved { peer_id: 1 });
//! if let ProcessResult::Rejected { admissible, .. } = result {
//!     // admissible: the set of commands valid in the current state.
//!     // The caller can use this to steer its protocol loop.
//!     assert!(admissible.contains(&Command::ReadyObserved { peer_id: 2 }));
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
//! machine.process(Command::ReadyObserved { peer_id: 3 });
//!
//! // Quorum of 4 reached → Bootstrapped.
//! let result = machine.process(Command::ReadyObserved { peer_id: 4 });
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
//!                        ↓
//!                     TimedOut
//! ```
//!
//! | State | Carries |
//! |---|---|
//! | `Initial` | Nothing — unit struct |
//! | `Pinging` | Active pinging and collecting peer sets |
//! | `Collecting` | Collecting and pinged peer sets |
//! | `Bootstrapped` | Terminal — quorum reached |
//! | `TimedOut` | Terminal — deadline expired before quorum |
//!
//! Terminal states are truly terminal: once reached, the machine rejects
//! every command other than `Probe`. The compiler can't enforce this, but
//! our test suite can — and does.
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
#![deny(unsafe_code)]

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

pub use types::PeerId;
