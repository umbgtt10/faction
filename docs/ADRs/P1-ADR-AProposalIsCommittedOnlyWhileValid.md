# P1-ADR-AProposalIsCommittedOnlyWhileValid

- **Status:** Accepted
- **Date:** 2026-11-17
- **Priority:** P1 (derived from `P0-ADR-ProtocolAgnostic`)
- **Phase:** 6

## Context
A coordinator decides a change on its own view when it is asked for it -- the addition of a peer that is not a
member, the removal of a member its quorum can spare -- and the layer above commits it later. Through the
specification's 0.8.0 nothing said the layer above must check it again. Changes already ran concurrently across a
cluster, one per coordinator: two removals each sparing the quorum on its own could commit together below the
required count, and an addition and a removal of one peer could both commit, the later winning. A change was safe
only because the layer above checked it again at commit time, unstated: the chaos tier's did, and every real consumer
had to write its own.

## Decision
A proposal is decided at the epoch its answer's view shows. The specification states what the layer above owes it:
to commit it only while no change committed since that epoch conflicts with it -- one naming the same peer, or, for a
removal, another removal or a new required count. A proposal it declines it reports to the coordinator,
`ProposalDeclined`, carrying the epoch it declined it at, once it has reported every commit up to that epoch; the
coordinator decides the change again on its present view. An addition is proposed again; a removal is proposed again
while the quorum can spare its member, and given up, `RemovalRefused`, once it cannot. A decline naming no proposal in
flight is answered `DeclineIgnored`. `ProposeAddition` and `ProposeRemoval` keep their form; the reference protocol's
proposals carry the epoch, for the caller to submit with the change.

## Forcing constraints / Evidence
The rule reads the committed changes alone, never a node's suspicions, so any layer above can keep it: Raft's leader
applying its log, IBFT's proposer building a block. A valid proposal holds at its commit what the node decided it on:
for an addition, its peer still no member; for a removal, the members at least those its quorum check counted, the
count unchanged. An epoch in the outcomes would have changed every released vector that proposes after a commit; the
view the answer carries states it already. A decline decided again by the machine keeps the newcomer's
acknowledgement: aborted and asked again, an addition would cost a round trip to the newcomer once more. Under the
layer above's obligations a declined addition never reaches a node still holding it -- the commit it conflicted with
names its peer, and ends the change -- so a declined addition is always proposed again; a declined removal is decided
again on the suspicions the node holds now.

## Rejected alternatives
**Strict: valid only if nothing at all has committed since.** Simpler to keep, but it serializes what ran in parallel:
two additions of different peers, from two coordinators, one of them waiting a round. A layer above may still be
strict -- it may decline more than the rule asks, never commit more.

**The layer above aborting a declined proposal (`AdditionAborted`, `RemovalAborted`), the caller asking again.**
Rejected for the newcomer's round trip.

**An epoch in `ProposeAddition` and `ProposeRemoval`.** Rejected: it would change released answers, and the view
carries it.

**A decline from ahead of the node's epoch refused.** Rejected: the node could do nothing with it but wait for the
commits, and its change would hang until aborted. Decided again on a view without them, a change is declined again
until they reach it.

## Consequences
The specification's 0.9.0: §17, the command `ProposalDeclined`, the outcome `DeclineIgnored` and four rows; every
vector 0.8.0 released answered alike. The chaos tier's layer above can commit by the rule alone and let proposals
race. Several changes in flight on one coordinator -- Phase 6's other half -- build on it.
