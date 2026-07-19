# P1-ADR-TerminalStatesAreNotSinks

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P1 (derived)
- **Phase:** 0-bugfix

## Context
A bootstrapping barrier that reaches a terminal state and then goes silent can
strand the very peers it was coordinating. A node that concluded — by quorum, or
by a premature deadline — stopped answering, so a peer that missed the original
readiness broadcast (a dropped message, a fast concluder) waited forever. Seen on
real hardware in two independent consumers.

## Decision
A concluded node is terminal but never a silent sink. `Bootstrapped` — the only
terminal state — still accepts `ParticipationObserved` and re-advertises its
readiness (`AcknowledgeRejoin`); a missed deadline is a non-terminal
`DeadlineMissed` fact, not a `TimedOut` dead-end.

## Forcing constraints / Evidence
Liveness: a peer's only path to learn a concluded node's readiness is that node
answering. Readiness was disseminated push-with-retry, but the retry was bounded
by the *sender's* conclusion, not the *receiver's* need — so a fast concluder
that cancelled its retry stranded a straggler. The `(size, quorum)` convergence
sweep reproduces it deterministically at `(2, 2)`; the late-arrival sweep
reproduces the deadline dead-end.

## Rejected alternatives
Keeping `TimedOut` as a receptive state — duplicates `Collecting`'s quorum logic,
and a receptive "concluded" state is a contradiction. A bootstrapped node
re-broadcasting on a timer forever — unbounded chatter, no stop condition.
Responding to `ReadyObserved` as well as pings — two bootstrapped nodes would
ping-pong their re-advertisements forever; reacting only to a *ping* is
self-limiting, since a peer pings only while still trying.

## Consequences
`Bootstrapped` admits `ParticipationObserved`; `DeadlineExpired` is repeatable
and non-terminal; `Conclusion` and the internal `TimedOut` state collapse to
`Bootstrapped` only, with `PeerState::TimedOut` surviving as a derived view flag
off `ClusterView::deadline_missed`. Breaking API changes (new outcomes, dropped
`Conclusion::TimedOut`); they ship in 0.4.0 alongside Phase 1 (dynamic joining),
not as a standalone release. Consumer reconnect workarounds collapse toward a
forwarding shim.

## Enforcement
`dropped_ready` `(2, 2)` proves a bootstrapped node re-advertises to a stranded
peer; `late_arrival::cluster_recovers_after_deadline_via_late_readiness` proves a
timed-out cluster recovers; a property test asserts a missed deadline never
concludes. All run in the stage-1 gate.
