# P2-ADR-TotalObservability

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context
The machine is a debugging and audit surface for distributed lifecycle
events. If any input could pass through `process` without being reported,
the observability story would have a hole exactly where the
hard-to-reproduce bugs live — the rejections and no-ops, not the happy path.

## Decision
Every command reaching `process` is reported to the injected `Observer`
exactly once, on every path: `observe` for an accepted transition,
`observe_query` for a `Probe`, `observe_rejection` for a rejected command.
The `Observer` is a channel distinct from the `Outcome`s returned to the
caller — outcomes say *what the caller must do*, observations say *what
happened*.

## Forcing constraints / Evidence
Observability that covers only the state-changing path is worse than none —
it hides the queries and rejections where lifecycle bugs actually hide.
Giving each of the three `process` exit paths its own `Observer` call makes
total coverage structural rather than something to remember per feature.

## Rejected alternatives
Fold observability into the returned `Outcome`s (every caller opts into its
own logging), or observe only state-changing transitions. Rejected: the
first couples every caller to observability wiring; the second reintroduces
the coverage hole for queries and rejections.

## Consequences
`Observer` has one method per `process` exit (`observe`, `observe_query`,
`observe_rejection`); `NoOpObserver` is provided for the "just drive it"
case. New commands and states inherit coverage for free — they flow through
the same three exits.

## Enforcement
The three exit paths in `process` each call an `Observer` method;
`core/tests/observer_tests.rs` asserts observation on the transition, query,
and rejection paths.
