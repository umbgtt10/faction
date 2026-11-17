# P1-ADR-ANewcomerHoldsItsMembershipProvisionally

- **Status:** Accepted
- **Date:** 2026-11-16
- **Priority:** P1 (derived from `P1-ADR-TheEpochComesWithTheCommit`)
- **Phase:** between 5 and 6, found by the chaos validation

## Context
A newcomer is created from the membership its coordinator offers it, and itself, at the coordinator's epoch: it holds
a member, itself, that no commit has added yet. Through specification 0.5.1 nothing told it apart from a member
holding a committed membership, so it answered a claim from behind as any node does, sending the peer its membership.
The chaos validation's first soak found what follows: the coordinator restarted, its request lost and the newcomer's
addition never committed, pinged the newcomer from behind, and adopted the newcomer's membership -- a member the log
never committed, at an epoch the log had already numbered otherwise. Nothing ever corrected it.

## Decision
The configuration states whether the node's membership is provisional, and the node holds it and shows it in its view,
each omitted when it is not. A node whose membership is provisional refuses a claim from behind as any node does,
`StaleEpochIgnored`, and sends nothing: a member that holds a committed membership will. Its membership stops being
provisional, once and for all, when it applies the commit of its own addition, when it adopts a membership, and when it
is removed. The reference protocol creates every newcomer provisional.

## Forcing constraints / Evidence
The chaos soak's seed 1794855532 fails without the rule and passes with it, and is recorded among the chaos
regressions. A membership's epoch is the layer above's number for a committed membership: a membership at that number
holding another member breaks what the epoch is for.

## Rejected alternatives
**The newcomer created at epoch 0.** It would never be ahead of anyone, so never answer a claim from behind; but its
acknowledgement would then be from behind its coordinator, which refuses it, and its addition could never be proposed.

**The protocol dropping the `SendMembership` outcome.** The answer would be the specification's and the behaviour not:
`SendMembership` tells the caller to send, and a conforming caller does.

**Waiting for the Notary.** Whether a membership is genuine is the caller's to check (`P1-ADR-TheEpochComesWithTheCommit`),
but the membership here is the node's own, sent at its own word.

## Consequences
Specification 0.6.0 grows by the rule of §1: every released vector is answered alike. `Config::new` and `ClusterView::new`
take the flag, which every caller now passes. A provisional node that coordinates an addition still offers its membership
to its newcomer, as `AdditionRequested` asks; whether a provisional node may coordinate at all is left open.

## Enforcement
Specification 0.6.0: three rows of the matrix -- a claim from behind before completion and after it, and an
acknowledgement from behind, at a provisional node -- vectors 109 to 113, the independent model answering each before
the crate runs it, and the chaos regressions.
