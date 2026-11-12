# P2-ADR-ClusterViewBuilderAndDto

- **Status:** Accepted; amended 2026-11-12 — private fields behind one resolving `new`
- **Date:** 2026-07-20
- **Priority:** P2 (structural)
- **Phase:** 1

## Context
`ClusterView` served two masters. Internally, each `State::cluster_view(&previous)`
builds it by copy-and-override — a `with_peer_state` / `with_pinging_peers` / …
chain off a `new(...)` seed, threading the read-model forward one transition at a
time. Externally, the same type is what `Protocol::cluster_view()` hands consumers
as the public read-model (`P2-ADR-TotalObservability`). So the value a consumer
held also carried the `with_*` builder methods — a read-model offering the very
mutation it must not. Phase 1 (dynamic joining) forced the issue: the join system
tests needed to observe live membership on the view, and the interim guard — "it is
a VIEW: getters only, no mutations" — was a coding-standard stopgap for a type that
had not been split.

## Decision
The value consumers receive is an immutable read-model; construction lives in a
separate `ClusterViewBuilder`. The states assemble through the builder
(`new` / `from_view` / `with_*` / `build`) and finalize into the view at the
Faction boundary (`Protocol::cluster_view()` returns the view). `ClusterView` has
private fields, read getters — including the observable `members` — and one
constructor, `new`, which resolves a missed deadline into `TimedOut`; it carries no
`with_*` and no mutating surface. Both types are `pub`; the single invariant is
**consumers are handed the view, never the builder.**

## Forcing constraints / Evidence
`P2-ADR-TotalObservability` makes `ClusterView` the public observation surface;
"observations say *what happened*" is undermined if the held value can also
synthesize or rewrite observations through `with_*` or through its fields. Building
must therefore live off the boundary type. The view's one piece of derived state — a
missed deadline surfacing as `TimedOut` before bootstrap — is resolved in `new`, so no
`ClusterView`, however it was built, carries fields that disagree with each other.
Phase 1 made membership authoritative, observable state
(`P1-ADR-ConfigIsImmutableGenesis`), which had to appear on the view — and a
read-model gaining new observable state is exactly when its boundary shape has to be
pinned. The house rules settle the field visibility: `ClusterView` has behaviour
(`conclusion`, `is_concluded`), so `cargo stern4rust`'s `records-and-objects` makes
it an object with private fields, out of reach of the builder in its own file; the
builder therefore goes through a constructor, and `public-or-private` makes that
constructor `pub` rather than crate-only.

## Rejected alternatives
Keep one dual-hat type plus a "don't mutate the view" convention — a reminder, not a
structural guarantee; the `with_*` methods would still sit on the consumer's value.
Hide the builder behind visibility — encapsulation is not the point; both types are
`pub`. Public fields — a held view could be rewritten field by field, and a literal
could pair a missed deadline with a state other than `TimedOut`. A pure record with
no methods, moving `conclusion` and `is_concluded` onto `PeerState` — rewrites every
read site across the consumers for no behavioural gain.

## Consequences
`ClusterViewBuilder` (`core/src/cluster_view_builder.rs`) owns
`new` / `from_view` / `with_*` / `build`, used by all four states and Faction's base
seed; `build` hands its fields to `ClusterView::new`. Consumers read the view through
its getters and gain membership visibility through `members` (a `Members` value
object). A consumer that needs a view of its own — a test fixture, a simulation —
builds it through the builder or `new`, never by literal. The cost is a
seven-argument constructor that only the builder needs to call by hand.

## Enforcement
`core/tests/cluster_view_builder_tests.rs` drives the builder: each `with_*` updates
only its field, `from_view` copies every field, `members` is exposed on the built
view, and a deadline-miss resolves to `TimedOut` at `build()`.
`core/tests/cluster_view_tests.rs` checks that `new` stores every field, resolves a
missed deadline unless bootstrapped, and answers `conclusion` and `is_concluded`.
`cargo stern4rust` holds `records-and-objects` and `public-or-private` over `faction`
in stage 2.
