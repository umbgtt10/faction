# P2-ADR-CanonicalModelBuiltAgainstJoiningBranch

- **Status:** Proposed
- **Date:** 2026-07-24
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context

Phase 1 (dynamic joining) and this cross-cutting specification effort were
proposed together in one brief, raising a sequencing question: design the
canonical model against a pre-joining baseline and extend it later, or
design it directly against the joining vocabulary as it stands now.

## Decision

The canonical model is designed against `Command`/`Outcome` as they exist
with joining already included — there is no smaller, pre-joining baseline
left to target.

## Forcing constraints / Evidence

`Command`/`Outcome` already carry the full join vocabulary
(`JoinRequested`, `JoinApproved`, `JoinRejected`, `EmitJoinRequest`,
`MemberAdmitted`, `JoinDenied`, `DuplicateMemberIgnored`,
`AcknowledgeRejoin`) as load-bearing, not experimental, variants.
`ROADMAP.md` states Phase 1's status as `✅ Complete`. There is no earlier
commit or branch state this specification work would target instead that
isn't already superseded.

## Rejected alternatives

**Design the canonical model against a pre-joining baseline, extend it
once joining is confirmed stable.** Rejected: joining is not provisional —
it is complete, tested, and the current state of `main`. Designing against
an earlier baseline would mean redoing the canonical-model mapping work a
second time for no benefit, against a state that no longer exists.

## Consequences

The canonical model's first version already includes join-related states,
commands, and effects — there is no "joining not yet in scope" caveat to
carry through the specification. Any future phase (2 through 6) that adds
new commands/outcomes extends this same canonical model incrementally,
using the same discipline established by
`P1-ADR-CanonicalModelSeparateFromCoreTypes`.

## Enforcement

N/A — this is a scoping decision, not an ongoing invariant to check.
