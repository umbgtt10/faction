# P2-ADR-TotalObservability

- **Status:** Accepted; amended 2026-11-12 — one report per call, the command and its result
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
exactly once, on every path, by one method: `observe(&command, &result)`,
with the command and the `ProcessResult` it was answered — `Accepted`,
`Rejected` or `Probed` — by the time the result is returned. The observer is
told and answers nothing. It is a channel distinct from the result returned to
the caller: the outcomes say *what the caller must do*, the report says *what
happened*, and a node's reports, in order, are its log.

## Forcing constraints / Evidence
Observability that covers only the state-changing path is worse than none —
it hides the queries and rejections where lifecycle bugs actually hide. One
report after the result is decided, whichever result it is, makes total
coverage structural: there is no exit path to forget. And a report that is the
result itself is a log that replays — the next view, the outcomes and the
admissible commands all in it (`spec/SPEC.md` §3) — where a report of less
could only be read.

## Rejected alternatives
Fold observability into the returned `Outcome`s (every caller opts into its
own logging), or observe only state-changing transitions. Rejected: the first
couples every caller to observability wiring; the second reintroduces the
coverage hole for queries and rejections. One method per exit path — a
transition, a query, a rejection. Rejected: an accepted command's report lacked
the admissible commands, so its log could not be replayed whole, and each new
path would need a method of its own.

## Consequences
`Observer` has one method, taking the command and the result by reference;
`NoOpObserver` is provided for the "just drive it" case. New commands and
states inherit coverage for free — whatever they are answered is reported. The
view a command found is the one the report before it carried; the first
report's is the fresh node's.

## Enforcement
`Faction::process` decides the result, reports it, then returns it;
`crates/core/tests/faction_tests.rs` asserts that an accepted command, a probe and a
rejected command are each reported once with the result returned, and the
conformance tests assert that every vector's calls are reported with the
results answered, in order, and nothing else.
