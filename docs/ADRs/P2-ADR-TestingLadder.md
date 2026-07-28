# P2-ADR-TestingLadder

- **Status:** Accepted
- **Date:** 2026-07-20
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context
`faction` claims to be *exhaustively tested* — a load-bearing part of its identity,
not a slogan. That claim is only as good as the discipline behind it: one test style
catches one class of failure and misses others. A transition matrix proves every
`(state, command)` is defined but says nothing about convergence over a real socket;
a system test proves convergence but cannot enumerate the state space. Without a
named, enforced ladder the tiers drift — a feature lands with a unit test and skips
the matrix, or a property regresses unnoticed.

## Decision
Correctness rests on a fixed ladder of test tiers, each with a distinct obligation,
all run by one gate:

1. **Unit** — each type and each `*Step` in isolation (`core/tests/**`: `states/`,
   `config_tests`, `quorum_policy_tests`, `cluster_view_tests`, …).
2. **Exhaustive transition matrix** — every `(state, command)` cell is a defined
   transition (`P1-ADR-TotalityExhaustiveMatrix`) plus the admissible-command
   invariant (`core/tests/transition_matrix/`).
3. **Property-based** — invariants and a reference model replayed over random command
   sequences with `proptest` (`core/tests/property_tests/`).
4. **Cluster simulation** — deterministic multi-node convergence over in-memory
   transport/timer (`core-validation`, `protocol-validation`).
5. **System** — end-to-end over the real transports (in-memory, channels, TCP, gRPC)
   and spawn models (task, thread, process) (`system-tests`).

"Tested" means all five tiers pass under the gate; "done" means the gate is green.

## Forcing constraints / Evidence
Each tier is load-bearing because it is the only one that catches its class of
failure: the matrix proves totality but not liveness; property tests find invariant
violations no fixed case enumerates; the simulation and system tiers expose transport-
and timing-dependent bugs that pure-logic tiers cannot — the TCP partial-write hang
was a system-tier find, invisible to every tier above it. Binding all tiers to a
single gate makes coverage structural rather than per-feature memory.

## Rejected alternatives
Lean on the exhaustive matrix alone — proves every transition is defined, proves
nothing about convergence under real I/O. Ad-hoc tests without named tiers — coverage
gaps drift in silently. No gate / manual runs — tiers get skipped under deadline
pressure, which is exactly when they matter.

## Consequences
A new command or state must land in every applicable tier — unit, matrix,
admissible-invariant, and the property model — and a system scenario when it changes
runtime behaviour. The gate runs sequentially and is slower than unit-only, and the
system tier spawns processes and opens real sockets; that cost is the price of the
"exhaustively tested" claim.

## Enforcement
`scripts/run_stage_1.ps1` runs formatting, `clippy -D warnings`, the `no_std` check,
and tiers 1–5 (system tests single-threaded); `scripts/run_stage_2.ps1` runs the CRAP
complexity gate over `faction` and `faction-protocol`. Both green is the definition
of done.
