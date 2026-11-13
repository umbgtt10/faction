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

1. **Unit** — each type and each `*Step` in isolation (`crates/core/tests/**`: `states/`,
   `config_tests`, `quorum_policy_tests`, `cluster_view_tests`, …).
2. **Conformance** — every vector of the specification run against the specification's
   independent model, each deciding by exactly the rows it names, then against the crate:
   each step's result, the final answer, and every call reported to the observer as it was
   answered (`validation/ladder/tests/conformance/`, `P0-ADR-SpecificationIsNormativeCrateIsReference`,
   `P1-ADR-IndependentModelOfTheSpecification`).
3. **Exhaustive transition matrix** — every `(state, command)` cell is a defined
   transition (`P1-ADR-TotalityExhaustiveMatrix`) plus the admissible-command
   invariant, each case checked against the row of `spec/matrix.json` it names
   (`validation/ladder/tests/matrix/`).
4. **Property-based** — invariants, and the crate against the specification's
   independent model, replayed over random command sequences with `proptest`
   (`validation/ladder/tests/property/`).
5. **Replay** — a node's log, its configuration then every call it reported as the
   specification writes a step, replays whole into a node created from the
   configuration: for random command sequences, and for every node of a simulated
   cluster; a log altered, cut, reordered or misconfigured does not
   (`validation/ladder/tests/replay/`).
6. **Cluster simulation** — deterministic multi-node convergence over in-memory
   transport/timer (`core-validation`, `protocol-validation`).
7. **System** — end-to-end over the real transports (in-memory, channels, TCP, gRPC)
   and spawn models (task, thread, process) (`system-tests`).

"Tested" means every tier passes under the gate; "done" means the gate is green.

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
A new command or state lands in the specification first — its rows and its vectors —
then in the specification's model, and in every applicable tier: unit, matrix and
admissible-invariant, and a system scenario when it changes runtime behaviour. The gate is slower than unit-only, and the system tier spawns
processes and opens real sockets; that cost is the price of the "exhaustively tested"
claim.

## Enforcement
`just stage1` runs formatting, `clippy -D warnings` over every target, the `no_std`
check against a bare-metal target, and every tier — the system tier under `slotgate`,
each test in a process of its own with ports of its own. `just stage2` runs the house
gates, among them 100% line, region and function coverage of `faction` and
`faction-protocol` measured with the ladder's tiers running, and of `faction` again
measured with the conformance tier alone
(`P1-ADR-NoLineTheSpecificationDoesNotAskFor`). Both green is the definition of done.
