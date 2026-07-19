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
Totality is the property; the exhaustive matrix is its proof. `accept()`
defaulting to true, per-state overrides, and the `Rejected` result together
achieve totality structurally rather than by convention.

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
`core/tests/transition_matrix/state_transition_matrix_tests.rs`, gated by the
rule that no phase begins until its `(state, command)` coverage is 100%.
