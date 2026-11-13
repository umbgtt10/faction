# P1-ADR-RemovalThroughTheLayerThatCommits

- **Status:** Accepted
- **Date:** 2026-11-13
- **Priority:** P1 (derived from `P0-ADR-ProtocolAgnostic`)
- **Phase:** 4

## Context
Phase 2 suspects a member and never removes it: a suspect stays a member, and the
quorum is counted with it. Phase 3 grows a concluded cluster by one through the
layer that commits changes above it. A cluster must also shrink — a member gone
for good, a node retired — without a removal taking it below the count its
quorum needs. `faction` cannot commit anything (`P0-ADR-ProtocolAgnostic`,
`P0-ADR-PureMealyNoIO`): whatever orders the change, and whoever decides that a
suspect should go, is the caller's.

## Decision
A concluded cluster shrinks by one through the layer that commits changes above
it, coordinated as an addition is (`P1-ADR-AdditionThroughTheLayerThatCommits`)
but with no handshake: the node asked for a removal tells the caller to propose
it at once (`ProposeRemoval`) and is `Reconfiguring`, one change in flight of
either kind, its view's reconfiguration the removal. It refuses
(`RemovalRefused`) the removal of a member not suspected when, without it, the
members not suspected would number fewer than the required count; a suspect's
removal it never refuses for the quorum. Every node, in every stage, applies the
commit the layer above reports (`RemovalCommitted`): the peer leaves every list
the node holds, its liveness with it. A node whose own removal commits is
`Removed`, a sixth stage that admits a probe alone.

## Forcing constraints / Evidence
The member most worth removing is the one that cannot answer, so a removal waits
for no acknowledgement. A suspect's removal leaves the members not suspected as
they are, so refusing it would refuse the case removal exists for; a member not
suspected is one the quorum counts, so its removal is checked against the count
— on the coordinator's own view, as each node judges liveness for itself
(`P1-ADR-LivenessReportedNotStaged`). A commit is a fact of the layer above, true
on every node at once, so it is never refused: on a node that still hears the
member, it may lose the quorum, and says so. A node's own removal must end its
part in the cluster — it would otherwise go on probing, suspecting and holding a
quorum that counts it no more — and no member waits on it, since every member
applies the same commit.

## Rejected alternatives
**A handshake with the member removed.** Rejected for the first constraint.

**`NodeAlreadySuspected` as a reason to refuse.** Rejected: it refuses the
removal of a suspect, which the roadmap's own table made valid.

**A `RejectRemove { reason }` outcome.** Rejected: a removal in flight already
is the stage, and `Rejected` says it, as for a second addition; the one reason
left, the quorum, is `RemovalRefused`.

**Refusing every removal while the quorum is lost.** Rejected: a suspect's
removal does not make a lost quorum worse, and with a lowered count it is how a
cluster that lost members recovers.

**A second view field, `removal`, beside `addition`.** Rejected: two fields for
the one change a node holds, and a pair of values the rules never reach that
every reader must still handle. The view's addition became its reconfiguration.

**A node ignoring its own removal.** Rejected for the third constraint.

**Faction deriving the count from the members.** Rejected (`P0-ADR-ProtocolAgnostic`):
the right count is the protocol's — a third of faults, a majority of voters —
and the caller reports it, with `RequiredCountChanged`, when the change it
belongs to commits; lowered first, it shows no quorum lost between the two.

## Consequences
The specification's 0.4.0 carries three commands, four outcomes, the sixth
stage and its peer state, the view's reconfiguration in place of its addition,
ten rows and four widened; the 0.3.0 vectors change in that field and in the
admissible lists alone. `Removed` keeps what the node held but itself and its
liveness, and is never shown `TimedOut`. A way back for a removed node, and a
removed peer's stale identity, are Phase 5's (epochs and rejoin). The protocol
carries no message for a removal: the commit reaches each node from the layer
above, and a removed member's late answers are a non-member's.

## Enforcement
The specification's rows — `removal-non-member`, `removal-refused`,
`removal-requested`, `removal-aborted`, `commit-non-member`, `node-removed`,
`removal-completion-loses-quorum`, `removal-completed`, `removal-loses-quorum`,
`removal-committed` — and the vectors exercising them, answered by the
independent model and the crate alike; the matrix tier's cells for the sixth
stage and the three commands; the property tier's invariant that a removal is
proposed exactly when the quorum can spare the member; the in-process cluster's
seven scenarios; and five system scenarios in every spawn and transport.
