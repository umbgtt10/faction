# P1-ADR-SeveralChangesInFlight

- **Status:** Accepted
- **Date:** 2026-11-17
- **Priority:** P1 (derived from `P0-ADR-ProtocolAgnostic`)
- **Phase:** 6

## Context
Through Phase 5 a coordinator held one change at a time: `Reconfiguring` refused a second request of either kind, and
growing a cluster by many nodes went one addition per coordinator per round of consensus. Changes already ran
concurrently across a cluster, one per coordinator, and with `P1-ADR-AProposalIsCommittedOnlyWhileValid` the layer
above commits each only while it is still valid -- so the one-at-a-time limit on a coordinator guarded nothing the
rule does not.

## Decision
The configuration states how many changes the node may hold in flight at once as a coordinator, its bound,
`changes_in_flight`, 1 unless it states another and omitted at 1. A node with changes in flight fewer than its bound is
`Coordinating`, a new stage: it admits what `Reconfiguring` admits and both requests; a node with as many is
`Reconfiguring`, which admits no request, as before. The stage the changes in flight make -- none `Bootstrapped`, fewer
than the bound `Coordinating`, as many `Reconfiguring` -- is the matrix's next stage wherever a change starts or ends.
No two changes in flight name one peer: a request naming a peer with a change in flight is `DuplicateChangeIgnored`.
The removals in flight are counted together: the quorum must spare a member with every other member whose removal is
in flight gone too, requested or declined. Each change ends on its own; an adopted membership gives up every change,
one `ChangeAbandoned` each, in the order requested; no admission goes around any change in flight. The view lists the
changes in flight, `reconfigurations`, re-encoded once from the single `reconfiguration`.

## Forcing constraints / Evidence
The specification grows by its rule: with the bound at 1 no node is ever `Coordinating`, and every vector 0.8.0
released is answered alike. A queue of changes inside the machine (the ROADMAP's draft: `EnqueueAdd`, `Dequeued`,
`QueueFull`, `FlushQueue`) would order what the layer above already orders, and decide nothing the coordinator's own
checks and the commit rule do not. Counting removals together keeps the coordinator from proposing, alone, a pair the
quorum cannot spare -- the case the commit rule would decline after the fact.

## Rejected alternatives
**The queue.** Dropped, at the owner's word: ordering is the layer above's.

**`Reconfiguring` admitting requests below the bound, no new stage.** Rejected: a stage's admissions would depend on
its contents, and the admissible commands a caller reads would no longer say whether it can ask for another change.

**The list beside the single `reconfiguration`.** Rejected: two members saying one thing; a re-encoding, once, changes
no answer.

## Consequences
The specification's 0.9.0: the bound, the stage, the outcome, the list, rows and vectors for each, and every released
vector re-encoded once more. `P1-ADR-AdditionThroughTheLayerThatCommits` and
`P1-ADR-RemovalThroughTheLayerThatCommits` are amended: a coordinator holds up to its bound of changes, not one.
`Coordinating` wraps `Reconfiguring` (`P2-ADR-StateStructsBehindStageEnum`), and `InFlight` holds the changes. The
chaos tier draws each peer a bound of one to three, so several changes race on one coordinator as well as across
coordinators.
