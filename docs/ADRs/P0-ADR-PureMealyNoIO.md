# P0-ADR-PureMealyNoIO

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P0 (axiom)
- **Phase:** cross-cutting

## Context
The machine coordinates distributed lifecycle events but must run unchanged
across hosts with nothing in common — bare-metal microcontrollers and cloud
processes. Transport, timers, storage, and clocks differ on every one.

## Decision
The machine performs no I/O and has no side effects. It emits `Outcome`s
describing what should happen; the consumer executes them. `output =
F(state, input)`, a pure function.

## Forcing constraints / Evidence
The only thing portable across every target is the decision itself. A
primitive that owned transport or storage would have to abstract each per
host, and would forfeit determinism the moment it did.

## Rejected alternatives
The machine sending its own messages or writing its own state. Rejected:
couples the primitive to a host and destroys testability and replayability.

## Consequences
Every effect is a value the consumer runs: send this, persist this, reply to
that. Tests need no mocks — they assert on returned `Outcome`s. Persistence,
being I/O, therefore also lives outside the machine.

## Enforcement
`process` returns a `ProcessResult` and no I/O-capable dependency is injected
or reachable; `no_std` removes most of the I/O surface at compile time.
