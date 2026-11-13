# P1-ADR-SingleEntryPoint

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P1 (derived)
- **Phase:** cross-cutting

## Context
A consumer drives the machine from a protocol loop and must never need to know
the internal state graph to use it safely.

## Decision
A single method, `process(command) -> ProcessResult`, handles writes, reads,
and rejections uniformly. `ProcessResult` is one of `Accepted { outcomes,
cluster_view }`, `Rejected { cluster_view, admissible }`, or `Probed {
cluster_view, admissible }`. The machine is self-describing: on a probe or a
rejection it returns the set of commands admissible in the current state. The
load-bearing invariant is that a state's admissible set is exactly the
non-`Probe` commands its `step` answers, plus the always-available `Probe`:
`admissible_commands() == { c ≠ Probe : step(c) answers } ∪ { Probe }`. (`Probe`
is always admissible: it is read-only, and no stage steps on it, so it is answered
`Probed`.)

## Forcing constraints / Evidence
A self-describing entry point lets a consumer steer its loop by asking the
machine what is valid, instead of hard-coding the state graph. `Probe` is
read-only and reuses the same entry point rather than a parallel read API.

## Rejected alternatives
Separate methods per command, or a read API distinct from the write API.
Rejected: duplicates the state-gating logic in two places and lets them drift.

## Consequences
Consumers branch on three result variants. `Stage::admissible_commands()` is
computed from what `step` answers rather than declared beside it, so a state's
self-description cannot drift from what it steps on.

## Enforcement
Every transition's admissible set is asserted against its expected value in
the exhaustive `(state × command)` matrix, every result variant is exercised
there, and a dedicated invariant test
(`validation/ladder/tests/matrix/admissible_invariant_tests.rs`) checks
`admissible == { c ≠ Probe : c is not rejected } ∪ { Probe }` per state directly.
