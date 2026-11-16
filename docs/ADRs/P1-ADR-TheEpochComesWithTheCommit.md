# P1-ADR-TheEpochComesWithTheCommit

- **Status:** Accepted
- **Date:** 2026-11-16
- **Priority:** P1 (derived from `P0-ADR-ProtocolAgnostic`)
- **Phase:** 5

## Context
Through Phase 4 a node knew the members it held and nothing said which membership was newer than which. A node
restarted from its configuration held the genesis members after the cluster had changed; a removed node had no way
back; and a node that had missed a change -- restarted, cut off, or slower to apply a commit -- still signalled
participation and readiness, and was counted toward a quorum of a membership it did not hold. `faction` commits
nothing (`P0-ADR-PureMealyNoIO`); the changes it versions are the layer above's.

## Decision
The epoch, the version of the membership a node holds, comes with the commit: `AdditionCommitted`,
`RemovalCommitted` and `RequiredCountChanged` carry the layer above's own strictly increasing number for the change --
Raft's log index, IBFT's height -- and a node takes it, never counting. A commit at an epoch of one or more, at or
below the node's, is held already and ignored (`CommitIgnored`), before any other answer to it. A node's own
signals carry its epoch: a member's participation or readiness, or the awaited acknowledgement, from below the node's
epoch is refused (`StaleEpochIgnored`) and answered with the membership (`SendMembership`); liveness carries none. A
membership handed to the node (`MembershipReceived`) above its epoch is adopted in every stage -- naming the node, the
stage kept, a change in flight given up (`ChangeAbandoned`), a removed node taken back to `Initial`; not naming it,
the node removed. Epoch 0 is omitted wherever it would stand, and a caller that numbers nothing is answered as before.

## Forcing constraints / Evidence
Commits reach nodes at different moments, and a node may adopt a membership before the commit that made it reaches
it: a node counting the commits it applied would count that one twice, and two numbers -- faction's and the layer
above's -- would need reconciling. With the commit's own number, applying is idempotent. Refusing liveness from
behind would turn every commit's propagation into suspicion, and a slow member into a removed one; refusing claims
alone keeps a stale node from the quorum, which is the split the roadmap names. A node learns it is behind only by
being handed the membership, and adopts it in the same call, so staleness is reported, as liveness is
(`P1-ADR-LivenessReportedNotStaged`), never a stage.

## Rejected alternatives
**Faction counting applied changes.** Rejected for the first constraint.

**`SplitSuspected` and `Rejoining(PeerId)` stages** (the roadmap's draft). Rejected: a node behind is not a stage, and
rejoining is `Initial` and bootstrapping again, the stages a newcomer passes through.

**Refusing a non-member's signal from behind with the membership.** Rejected: a newcomer pinging before its admission
would be handed a membership that does not name it, and removed. A non-member's signal is ignored as before.

**The epoch of each node's last confirmed membership, kept per member.** Rejected: each signal carries its sender's
epoch, which says the same per message.

**`ChangeAbandoned` naming the change's peer.** Rejected: it carries the change as the view showed it, so the caller
knows which change, of which kind, at which step, it no longer has in flight.

## Consequences
The configuration, the view, three signals and three commits gain an optional epoch, and the specification its first
growth under its own rule: every vector released through 0.4.1 is answered alike, a released vector's admissible
commands compared among those its version names. `Command` is no longer `Copy`, since a membership carries its
members. Whether a membership is genuine is the caller's to check before reporting it -- the Notary bookkeeper's, once
built; faction adopts what it is handed. `P1-ADR-TerminalStatesAreNotSinks` is amended: `Removed` is left by
adoption alone.

The reference protocol carries the epoch on `Ping`, `Ready`, `Membership` and `MembershipAcknowledged`. A coordinator
inviting a newcomer sends its own membership, which does not yet name the newcomer, at its own epoch -- for a node
removed and invited back, usually the epoch it was removed at -- so a removed node would hold that membership already.
The owner chose the newcomer path: a removed node a membership invites is made a newcomer again by whoever runs it, and
joins from the membership and itself, as any newcomer does. `Removed` left by adoption serves a caller whose layer above
hands a removed node a later membership naming it.

## Enforcement
Specification 0.5.0: vectors 94 to 108 and the fourteen rows they exercise, the independent model answering every
vector before the crate runs one; the transition matrix's cases for each new row; and two properties over random
sequences, memberships and epochs among them -- a node's epoch never decreases, and a claim from behind never counts.
