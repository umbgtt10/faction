# P1-ADR-TotalityExhaustiveMatrix

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P1 (derived)
- **Phase:** cross-cutting

## Context
Lifecycle code fails at the edges — the "impossible" input arriving in the
"wrong" state. Partial handling (a panic, an `Err` on the unexpected) is
exactly where those bugs hide.

## Decision
Every `(state, command)` pair is a defined transition. Degenerate inputs are
first-class outcomes (`DuplicateParticipationIgnored`, `NonMemberIgnored`, …),
never errors and never panics. The `(state × command)` matrix tests every pair
explicitly, and the suite is a strict superset across phases — a Phase-N test
passes unchanged at Phase N+6.

## Forcing constraints / Evidence
Totality is the property; the exhaustive matrix is its proof. Every command is
offered to the stage's `step`, which answers it — or answers nothing for what the
stage refuses, and the command is `Rejected` — so totality holds structurally
rather than by convention.

## Rejected alternatives
Returning `Err` or panicking on unexpected input. Rejected: it converts a
defined behaviour into an error path, and error paths are the least-tested
code in the system.

## Consequences
New states and commands (Phases 1–6) must extend the matrix to stay exhaustive
before the phase is declared complete. One deliberate exception to
strict-superset: correcting a genuine Phase-0 bug rewrites that behaviour's
tests once (the terminal-state-sink fix — see
`P1-ADR-TerminalStatesAreNotSinks`) — a correction to a known-wrong baseline,
not a superset break.

## Enforcement
`spec/matrix.json` gives every `(stage, command)` its row, and the conformance
vectors exercise every row. `ladder/tests/matrix/state_transition_matrix_tests.rs`
names the row each case exercises and checks the crate's answer against it — the
result, the outcomes in order and the stage after — gated by the rule that no phase
begins until its `(state, command)` coverage is 100%.
