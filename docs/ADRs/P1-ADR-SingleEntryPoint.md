# P1-ADR-SingleEntryPoint

- **Status:** Accepted; amended 2026-11-17 — the answer a numbered receipt; amended 2026-11-18 — the admissible
  commands listed by kind
- **Date:** 2026-07-19
- **Priority:** P1 (derived)
- **Phase:** cross-cutting

## Context
A consumer drives the machine from a protocol loop and must never need to know
the internal state graph to use it safely.

## Decision
A single method, `process(command) -> Receipt`, handles writes, reads,
and rejections uniformly; the receipt carries the answer, its `output`, and its
number (`P1-ADR-TheLogIsTheRecord`). The output, a `ProcessResult`, is one of `Accepted { outcomes,
cluster_view }`, `Rejected { cluster_view, admissible }`, or `Probed {
cluster_view, admissible }`. The machine is self-describing: on a probe or a
rejection it returns the set of commands admissible in the current state, each by
its kind, a `CommandKind` -- the command's name without the peer or the epoch it
would carry, as the specification writes it; `Command::kind()` names a command's. The
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

## Amendment 2026-11-18: the admissible commands by kind
Through 0.12.0 the admissible commands were `Command`s, each carrying a placeholder
-- peer 0, epoch 0, no members -- that read as if peer 0 were meant. A generic peer id
has no 0 to put there, and the specification
never wrote one: it lists admissible commands by name. A result lists `CommandKind`s
instead, and a caller asks `admissible.contains(&command.kind())`.
