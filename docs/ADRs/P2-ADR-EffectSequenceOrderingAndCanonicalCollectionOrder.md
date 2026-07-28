# P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder

- **Status:** Proposed
- **Date:** 2026-07-24
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context

Vectors are only checkable if comparison is unambiguous. Two determinism
questions the brief leaves open need a stated answer: whether a command's
effects are compared as an ordered sequence or an unordered set, and
whether canonically-serialized collections (`Vec<PeerId>` peer sets) need
a defined sort order.

## Decision

**Effects are a sequence, order significant.** `State::step()` already
returns `Vec<Outcome>` — an ordered type — so sequence semantics require
no code change, only documenting the existing guarantee. **Collections
serialize in insertion order, not sorted.** `step()`'s purity already
makes insertion order fully deterministic and reproducible from a given
command sequence; canonical ordering is a documentation decision, not a
transformation to add.

## Forcing constraints / Evidence

Every `Outcome`-producing path in `core/src/states/` (e.g.
`LocalCompletionStep::new` pushing `LocalParticipationCompleted` then
`BroadcastLocalReady` then, conditionally, `Concluded`) already produces
outcomes in a meaningful, causally-ordered sequence — collapsing that to
set semantics would require every harness to implement order-insensitive
comparison for no actual benefit, since the order is already
deterministic and already meaningful. `Members`/`ClusterView`'s
`Vec<PeerId>` fields are populated in the order peers are observed, and
that order is a pure function of the input command sequence — already
reproducible, already comparable by deep equality.

## Rejected alternatives

**Effects as an unordered set.** Rejected: forces order-insensitive
comparison in every harness for a property (order) that's already
deterministic and already carries meaning.

**Sort collections by identifier before serializing.** Rejected: adds a
transformation step in the mapping layer for a property that's already
deterministic either way — marginal readability in a hand-inspected
vector file, no correctness benefit, and one more thing the mapping layer
has to get right.

## Consequences

Vector comparison is a simple deep equality check on both effects and
collections — no custom comparator needed in any harness, in any
language. The cost is that a vector file's collection ordering reflects
whatever order the `setup` command sequence happens to produce, which may
look arbitrary to a reader unfamiliar with that sequence, rather than a
tidier sorted presentation.

## Enforcement

None yet — enforced once `FORMAT.md` states both rules explicitly and the
harness gate checks vectors by deep equality with no custom ordering
logic.
