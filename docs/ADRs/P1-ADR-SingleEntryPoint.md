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
non-`Probe` commands it accepts, plus the always-available `Probe`:
`admissible_commands() == { c ≠ Probe : accept(c) } ∪ { Probe }`. (`Probe` is
always admissible because it is read-only and intercepted before `accept()`.)

## Forcing constraints / Evidence
A self-describing entry point lets a consumer steer its loop by asking the
machine what is valid, instead of hard-coding the state graph. `Probe` is
read-only and reuses the same entry point rather than a parallel read API.

## Rejected alternatives
Separate methods per command, or a read API distinct from the write API.
Rejected: duplicates the state-gating logic in two places and lets them drift.

## Consequences
Consumers branch on three result variants. The `State` trait's default
`admissible_commands()` returns every command, so each non-trivial state
carries a real obligation to override it in lockstep with `accept()` — or the
self-description lies. Known open inconsistency: `Accepted` does not carry
`admissible` (`OPEN_POINTS.md` §11).

## Enforcement
Every transition's admissible set is asserted against its expected value in
the exhaustive `(state × command)` matrix, and every result variant is
exercised there.
