// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

//! **A `no_std`, zero-`unsafe`, protocol-independent cluster membership state machine.**
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
//! **All six phases are built** -- bootstrapping, dynamic joining, single-node
//! addition, failure detection, single-node removal, epochs and rejoin, and
//! concurrent changes -- conformant to version 0.10.0 of the specification in
//! the repository's `spec/`.
//!
//! A peer id is any type implementing [`PeerId`](instance::peer_id::PeerId):
//! the unsigned integers, `u64` every type's default, or a type of the
//! caller's own once it says so. The machine copies peer ids and compares
//! them, and does nothing else with one.
//!
//! # What it does
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
//! | `Coordinating` | Concluded, with changes in flight fewer than its bound — shown as `Bootstrapped` |
//! | `Reconfiguring` | Concluded, with as many changes in flight as its bound — shown as `Bootstrapped` |
//! | `Removed` | Removed from its own cluster — admits a membership and a probe alone |
//!
//! `Bootstrapped` is the only conclusion. A missed deadline is recorded as
//! a fact — surfaced as `PeerState::TimedOut` — without leaving the current
//! state, so the node stays receptive and can still converge. And a
//! bootstrapped node answers a still-pinging peer with its readiness, so a
//! concluded node is never a silent sink.
//!
//! A concluded node grows or shrinks through the layer that commits changes
//! above it, up to its bound of changes in flight at once, 1 unless its
//! configuration states another: asked for an addition or a removal, it is
//! `Coordinating` while it can take another and `Reconfiguring` once it
//! cannot, each change in flight until the layer above reports its commit,
//! which every node applies, or its abort. No two changes in flight name one
//! peer. It refuses to coordinate a removal its quorum cannot spare: a member
//! not suspected, without whom and every member whose removal is in flight the
//! members not suspected would fall below the count. A node whose own removal commits is
//! `Removed`, and admits nothing but a membership and a probe.
//!
//! Every membership carries an epoch, the layer above's own number for the
//! change that made it, which a commit brings: a commit the node holds
//! already is ignored, a member's participation or readiness from behind is
//! refused and answered with the membership, and a membership above the
//! node's epoch is adopted in every stage -- a removed node it names coming
//! back as a fresh one. Liveness carries no epoch. A caller that numbers
//! nothing gives epoch 0, and is answered as before.
//!
//! A concluded node also tracks its members' liveness, the caller keeping the
//! clock: at each probe period it names the member to probe, round-robin; a
//! member that misses its probe is probed through others; one that misses as
//! many periods running as the configuration's suspicion threshold is
//! suspected, and one heard from again is revived. Its view says when the
//! members not suspected fall below the required count, and when they reach it
//! again. A suspect stays a member until its removal commits: liveness is
//! reported, never a stage.
//!
//! # Why faction
//!
//! You bring the network, the transport, and the definition of "ready."
//! `faction` brings the state transitions -- every single one of them tested,
//! observable, and replayable from an input log.
//!
//! * **Protocol-agnostic** — no opinion on what a peer *is* or how messages move
//! * **Deterministic** — same inputs → same outputs, always
//! * **Exhaustively tested** — every `(state, command)` pair covered by an explicit matrix
//! * **Zero unsafe** — `#![forbid(unsafe_code)]`
//! * **`no_std + alloc`** — runs on bare metal, WASM, embedded, and cloud
//!
//! # Example
//!
//! ```rust
//! use faction::command::Command;
//! use faction::command_kind::CommandKind;
//! use faction::config::Config;
//! use faction::faction::Faction;
//! use faction::instance::instance_id::InstanceId;
//! use faction::no_op_observer::NoOpObserver;
//! use faction::peer_state::PeerState;
//! use faction::process_result::ProcessResult;
//! use faction::quorum_policy::QuorumPolicy;
//!
//! extern crate alloc;
//!
//! // A 5-node cluster. We need 4 to agree before proceeding. Its peer ids are
//! // u64, every type's default; any type implementing `PeerId` will do.
//! let config: Config = Config::new(
//!     InstanceId { node: 0, incarnation: 1 }, // our peer id, its first incarnation
//!     alloc::vec![0, 1, 2, 3, 4],        // all peers
//!     QuorumPolicy::new(4),              // quorum threshold
//!     1,                                 // missed probe periods before suspicion
//!     0,                                 // the epoch of this membership
//!     false,                             // whether it is a newcomer's, provisional
//!     1,                                 // changes it may hold in flight at once
//! );
//!
//! let mut machine = Faction::new(config, Box::new(NoOpObserver));
//!
//! // Phase 1 — feed participation signals as they arrive from the wire. Every
//! // answer is a receipt: its output, numbered one more than the last.
//! let receipt = machine.process(Command::ParticipationObserved { peer_id: 1, epoch: 0 });
//! assert_eq!(receipt.number, 1);
//! assert!(matches!(receipt.output, ProcessResult::Accepted { .. }));
//! machine.process(Command::ParticipationObserved { peer_id: 2, epoch: 0 });
//!
//! // A duplicate is accepted but marked ignored — and, like every result,
//! // it reports the kinds of command admissible next.
//! if let ProcessResult::Accepted { admissible, .. } =
//!     machine.process(Command::ParticipationObserved { peer_id: 1, epoch: 0 }).output
//! {
//!     assert!(admissible.contains(&CommandKind::LocalParticipationCompleted));
//! }
//!
//! // Probe at any time — read-only, zero side effects.
//! if let ProcessResult::Probed { cluster_view, .. } =
//!     machine.process(Command::Probe).output
//! {
//!     assert_eq!(cluster_view.pinging_peers(), &[1, 2]);
//! }
//!
//! // Phase 2 — local participation done, now collecting readiness.
//! machine.process(Command::LocalParticipationCompleted);
//! machine.process(Command::ReadyObserved { peer_id: 1, epoch: 0 });
//! machine.process(Command::ReadyObserved { peer_id: 2, epoch: 0 });
//!
//! // Self plus peers 1, 2, 3 = 4 confirmations → quorum → Bootstrapped.
//! let result = machine.process(Command::ReadyObserved { peer_id: 3, epoch: 0 });
//! if let ProcessResult::Accepted { cluster_view, .. } = result.output {
//!     assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
//!     // The cluster is live. Hand off to the application.
//! }
//! ```
//!
//! # Observer
//!
//! Every call — an accepted command, a rejected one, a probe — is reported once
//! to the [`Observer`], with the command and its receipt: the result, numbered
//! one more than the last. Wire it to telemetry,
//! an audit log, or a test assertion: it is told and answers nothing, so the
//! machine doesn't care. [`NoOpObserver`] is provided for the common "just drive
//! the machine" case.
//!
//! # Siblings
//!
//! Faction is one of four bookkeepers a consensus protocol orchestrates, none
//! aware of the others: [`faction`](https://crates.io/crates/faction) keeps
//! membership, [`beggar`](https://crates.io/crates/beggar) recovery, `notary`
//! credentials (not yet built), and [`cartel`](https://crates.io/crates/cartel)
//! consensus. All are `no_std`, protocol-independent and specified, each with one
//! entry point and an observer.
//! Faction alone is a state machine: membership has genuine states a node moves
//! between, where the registries beside it have none.
//!
//! # Further reading
//!
//! * [README](https://github.com/umbgtt10/faction#readme) -- the overview: how it works, validation, project status
//! * [Specification](https://github.com/umbgtt10/faction/blob/main/spec/SPEC.md) -- normative: this crate is one implementation of it
//! * [Architecture](https://github.com/umbgtt10/faction/blob/main/docs/ARCHITECTURE.md) -- the design
//! * [ADRs](https://github.com/umbgtt10/faction/tree/main/docs/ADRs) -- the decisions, one per file

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod no_op_observer;

pub mod cluster_view;
pub mod cluster_view_builder;
pub mod command;
pub mod command_kind;
pub mod conclusion;
pub mod config;
pub mod faction;
pub mod instance;
pub mod members;
pub mod miss_count;
pub mod observer;
pub mod outcome;
pub mod peer_state;
pub mod process_result;
pub mod quorum;
pub mod quorum_policy;
pub mod reconfiguration;
pub mod stage;
pub mod state;
pub mod states;
pub mod types;

#[cfg(doctest)]
pub mod doctests;
