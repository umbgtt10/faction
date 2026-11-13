# P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder

- **Status:** Accepted
- **Date:** 2026-11-12
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context

Vectors are only checkable if comparison is unambiguous. Two determinism
questions need a stated answer: whether a command's outcomes are compared as an
ordered sequence or an unordered set, and whether the lists of peers a view
carries need a defined order.

## Decision

**Outcomes are a sequence, order significant.** `State::step()` already returns
`Vec<Outcome>` — an ordered type — so sequence semantics need no code change,
only the existing guarantee stated. **Lists of peers are in arrival order, not
sorted:** the members as the configuration gives them, then each admitted peer;
the pinging and the collecting peers in the order their signals were accepted.
`step()`'s purity already makes that order a function of the commands given, so
it is reproducible as it is (`spec/SPEC.md` §1).

## Forcing constraints / Evidence

Every outcome-producing path in `crates/core/src/states/` (e.g.
`LocalCompletionStep::new` pushing `LocalParticipationCompleted` then
`BroadcastLocalReady` then, conditionally, `Concluded`) already produces
outcomes in a meaningful, causally ordered sequence — collapsing that to set
semantics would make every harness compare without order for no benefit.
`Members` and `ClusterView`'s peer lists are filled in the order peers are
observed or admitted, and that order is a pure function of the configuration and
the commands: already reproducible, already comparable by deep equality.

## Rejected alternatives

**Outcomes as an unordered set.** Rejected: every harness would compare without
order, for a property that is deterministic and carries meaning.

**Peers sorted by id.** Rejected: the crate would sort what it already holds in
a reproducible order, and its consumers would see members and peers reordered,
for no correctness benefit: faction's peers arrive in an order of their own.

## Consequences

Vector comparison is deep equality on both outcomes and peer lists — no custom
comparator in any harness, in any language. A vector's peer lists follow its
commands, which may look arbitrary to a reader who expects them sorted.

## Enforcement

`spec/SPEC.md` §1 states both rules; the conformance tests compare each result
by deep equality, with no ordering logic of their own.
