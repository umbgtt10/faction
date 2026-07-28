# P2-ADR-ClusterViewBuilderAndDto

- **Status:** Accepted
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
The value consumers receive is a pure DTO; construction lives in a separate
`ClusterViewBuilder`. The states assemble through the builder
(`new` / `from_view` / `with_*` / `build`) and finalize into the DTO at the Faction
boundary (`Protocol::cluster_view()` returns the DTO). `ClusterView` is inert data —
public fields and read-only accessors, including the now-observable `members` — with
no constructor-style or mutating surface. Both types may be `pub`; the single
invariant is **consumers are handed the DTO, never the builder.**

## Forcing constraints / Evidence
`P2-ADR-TotalObservability` makes `ClusterView` the public observation surface;
"observations say *what happened*" is undermined if the held value can also
synthesize or rewrite observations through `with_*`. Building must therefore live
off the boundary type. The `build()` step also resolves the one piece of derived
state — a missed deadline surfacing as `TimedOut` — so the DTO is final data with no
behaviour, not a value whose fields could disagree with its accessors. Phase 1 made
membership authoritative, observable state (`P1-ADR-ConfigIsImmutableGenesis`), which
had to appear on the view — and a read-model gaining new observable state is exactly
when its boundary shape has to be pinned.

## Rejected alternatives
Keep one dual-hat type plus a "don't mutate the view" convention — a reminder, not a
structural guarantee; the `with_*` methods still sit on the consumer's value. Hide
the builder behind visibility — encapsulation is not the point; both types can be
`pub`. Drop the getters for pub-fields-only — that would rewrite every read site
across the workspace for no behavioural gain, so the fields are `pub` *and* the read
getters are retained; either way the DTO carries no construction or mutation surface.

## Consequences
A new `ClusterViewBuilder` (`core/src/cluster_view_builder.rs`) owns
`new` / `from_view` / `with_*` / `build`, used by all four states and Faction's base
seed. `ClusterView` becomes pub-field data with read getters and gains an observable
`members` (a `Members` value object); `build()` resolves the `TimedOut` derivation.
Consumers (IBFT/Raft over the path dependency) read the DTO unchanged and gain
membership visibility for free. The cost is a mechanical split and the mild
redundancy of getters coexisting with public fields.

## Enforcement
`core/tests/cluster_view_tests.rs` drives the builder: each `with_*` updates only its
field, `members` is exposed on the built view, and a deadline-miss resolves to
`TimedOut` at `build()`. `ClusterView` carries no `new` / `with_*` (construction
exists only on the builder), and the stage-1 gate compiles every read site against
the DTO surface.
