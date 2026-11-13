# P1-ADR-AdditionThroughTheLayerThatCommits

- **Status:** Accepted
- **Date:** 2026-11-13
- **Priority:** P1 (derived from `P0-ADR-ProtocolAgnostic`)
- **Phase:** 3

## Context
Phase 1 admits a peer at once, on each node alone: no coordination, no way back,
two nodes free to disagree about who is a member. A running cluster that grows
needs the opposite — every node counting the newcomer from the same committed
change, and never two changes racing. `faction` cannot commit anything: it has
no consensus, no I/O and no clock (`P0-ADR-ProtocolAgnostic`,
`P0-ADR-PureMealyNoIO`). Whatever orders the change is the caller's.

## Decision
A concluded node grows by one through the layer that commits changes above it.
The node asked for an addition coordinates it, one at a time: it is
`Reconfiguring` — still concluded, its peer state `Bootstrapped`, its view naming
the addition — hands the newcomer the membership (`SendMembership`), and tells
the caller to propose the addition (`ProposeAddition`) once the newcomer has
acknowledged it. Every node, in every stage, applies the commit the layer above
reports (`AdditionCommitted`); the coordinator's addition ends with it, or with
an abort (`AdditionAborted`). The required count changes by its own command
(`RequiredCountChanged`), reported with the commit it belongs to.

## Forcing constraints / Evidence
One addition at a time is a stage, so admission refuses a second structurally —
`Rejected`, the view naming the addition in flight — as it refuses everything a
stage does not admit (`P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors`).
A newcomer counted before it holds the membership could not take part; its
acknowledgement comes first. A commit is a fact of the layer above, true on every
node at once, so every node applies it whatever its stage — one still
bootstrapping, one restarting — and the newcomer is a member everywhere or
nowhere. A peer state of its own for `Reconfiguring` would make every caller that
reads `Bootstrapped` as concluded read a concluded node as not concluded.

## Rejected alternatives
**Additions in every stage.** Rejected: before bootstrap no layer above commits
anything yet; Phase 1's admission is the way in.

**The addition in flight as a flag inside `Bootstrapped`.** Rejected: admission is
by stage; a flag would need refusal rows of its own.

**A `RejectAdd { reason }` outcome for a second request.** Rejected: the one
reason is the stage, and `Rejected` already says it.

**Commits admitted on the coordinator alone.** Rejected: every other node would
fake an admission to learn the same fact.

**The required count fixed, or carried by the commit.** Rejected: a grown cluster
would keep its genesis count; a commit naming a member already admitted would have
to change the count and not the members.

## Consequences
The specification's 0.2.0 carries the fifth stage, five commands, six outcomes,
eleven rows and the view's `addition`; the 0.1.0 vectors gain the view's field and
each stage's new admissible commands, every outcome unchanged. `Reconfiguring`
refuses `JoinApproved`: no admission goes around an addition in flight. The
protocol carries the membership and its acknowledgement, re-sending the membership
until the acknowledgement comes; a `Newcomer` waits for its membership and becomes
a node of it. The system tests play the layer that commits, reporting the commit
to the newcomer first, so it never concludes on the old count.

## Enforcement
The specification's rows — `addition-member`, `addition-requested`,
`acknowledgement-ignored`, `acknowledgement-accepted`, `abort-ignored`,
`addition-aborted`, `commit-member`, `addition-completed`, `addition-committed`,
`count-concluded`, `count-set` — and the vectors exercising them, answered by the
independent model and the crate alike; the matrix tier's 65 `(stage × command)`
pairs; the property tier's random sequences, crate against model; the in-process
cluster's scenarios of a lost membership, a lost acknowledgement and a commit
reaching nodes still bootstrapping; and five system scenarios in every spawn and
transport.
