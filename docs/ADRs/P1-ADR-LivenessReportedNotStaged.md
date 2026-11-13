# P1-ADR-LivenessReportedNotStaged

- **Status:** Accepted
- **Date:** 2026-11-13
- **Priority:** P1 (derived from `P0-ADR-PureMealyNoIO` and `P0-ADR-ProtocolAgnostic`)
- **Phase:** 2

## Context
Once concluded, a node knew who its members were and nothing about whether they
were still there (L2). A consensus layer above needs to know when too few of them
answer, and a removal (Phase 4) needs to know which members are alive. `faction`
keeps no clock and performs no I/O (`P0-ADR-PureMealyNoIO`): it cannot time a probe
or send one. And it decides nothing on its own (`P0-ADR-ProtocolAgnostic`): who
leaves a cluster is the layer that commits changes, as who joins is
(`P1-ADR-AdditionThroughTheLayerThatCommits`).

## Decision
A concluded node runs SWIM's failure detector, the caller running its clock and its
wire. At each `ProbeDue` the node names the member to probe, round-robin; a member
that misses its probe is listed for indirect probes, the caller asking the first k;
one that misses as many periods running as the configuration's suspicion threshold
is suspected; one heard from again is revived. Liveness is something the view
reports — the suspected members, each member's missed periods, the member probed
last, and the quorum, `NotTracked` before the conclusion and then `Held` or `Lost` —
never a stage: it runs alongside bootstrapping and addition, as a missed deadline
does (`P1-ADR-TerminalStatesAreNotSinks`). Each node judges for itself; nothing is
gossiped. A suspected member stays a member; nothing is confirmed dead.

## Forcing constraints / Evidence
Three liveness stages would split `Reconfiguring` three ways and every admission row
with it, and a suspicion would change what a node admits, which nothing asks for.
Before the conclusion a node's peers are still starting, and probing then would
suspect every peer not yet up: the bootstrap's own signals and deadline are the
liveness that matters there. A random probe target, as in the SWIM paper, would make
two runs of the same calls answer differently (`P0-ADR-DeterministicReplayable`);
round-robin is SWIM's own deterministic variant, and probes every other member once
every N−1 periods. A suspicion gossiped would need incarnation numbers to be
refutable, and would make one node's view depend on another's.

## Rejected alternatives
**`Stable`, `Degraded` and `QuorumLost` as stages.** Rejected for the forcing
constraints above.

**The caller running its own detector and reporting only suspicion and revival.**
Rejected: every consumer would write the probing, and the specification would cover
none of it.

**A third level, `Down`, after a further timeout.** Rejected for now: nothing in
`faction` would act on it; Phase 4 can add it if its removal rules need one.

**Gossiped suspicion with incarnation numbers.** Rejected: incarnations belong with
epochs (Phase 5).

**k in the configuration.** Rejected: no row depends on it; the caller chooses how its
probes travel, as it chooses its timeouts.

## Consequences
The configuration gains the suspicion threshold, each node's own and fixed for its
life: a deployment sets how tolerant its nodes are through the period and timeouts,
the helpers asked, and the threshold. The specification's 0.3.0 carries four
commands admitted in the concluded stages alone, eleven outcomes, twelve rows of
liveness and five twins that say a crossing of the quorum, and the view's four
fields; the 0.2.0 vectors gain the threshold, the fields and the concluded stages'
admissible commands, every outcome unchanged. The protocol carries probes, their
answers and relayed probes, a helper keeping no table of what it relays.

## Enforcement
The specification's rows `probe-alone`, `probe-due`, `liveness-self`,
`liveness-non-member`, `revival-restores-quorum`, `peer-revived`, `alive-noted`,
`miss-of-a-suspect`, `probe-missed`, `suspicion-loses-quorum`, `peer-suspected`,
`miss-counted` and the quorum twins, and the vectors exercising them, answered by the
independent model and the crate alike; the matrix tier's liveness cells in every
stage; the property tier's invariants — every other member probed within one round,
a suspicion only at the threshold, the quorum held exactly while the members not
suspected reach the count; the in-process cluster's ten scenarios; and five system
scenarios in every spawn and transport.
