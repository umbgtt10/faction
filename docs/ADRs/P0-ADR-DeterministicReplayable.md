# P0-ADR-DeterministicReplayable

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P0 (axiom)
- **Phase:** cross-cutting

## Context
Lifecycle bugs are notoriously hard to reproduce. A machine whose behaviour
depends on wall-clock time, scheduling, or randomness can be neither replayed
nor exhaustively tested.

## Decision
The machine reads no ambient input — no clock, no RNG, no scheduler state. Its
entire output is a function of its explicit `(state, command)` inputs. The
same input sequence always reproduces the same state and outputs.

## Forcing constraints / Evidence
Determinism is what makes exhaustive `(state × command)` testing and
state-reconstruction-by-replay possible at all; neither guarantee survives a
single nondeterministic input.

## Rejected alternatives
Internal, clock-driven timeouts (the machine firing its own deadline).
Rejected: that is a nondeterministic input. Deadlines instead arrive as an
explicit `DeadlineExpired` command supplied by the consumer.

## Consequences
Timing is entirely the consumer's — "how long to wait" is never a `faction`
field; it arrives as a command or not at all. State can be rebuilt from an
ordered input log.

## Enforcement
No clock or RNG dependency exists in the crate; the transition matrix asserts
that identical input sequences yield identical outputs.
