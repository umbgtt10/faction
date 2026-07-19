# P2-ADR-StateAsTraitObject

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context
The machine must grow from Phase 0's handful of lifecycle states to many
more across Phases 1–6, without each addition destabilising the states that
already exist.

## Decision
Each state is its own struct implementing a single `State` trait (`step`,
`cluster_view`, `accept`, `admissible_commands`), held as `Box<dyn State>`.
A transition returns the next boxed state; the machine swaps it in. One
struct per file, under `core/src/states/`.

## Forcing constraints / Evidence
With per-state structs, adding a Phase-N state is adding a *file* — it
cannot edit the code of an existing state, so the "Phase-N tests pass
unchanged at Phase N+6" superset property is structurally supported rather
than merely hoped for.

## Rejected alternatives
A single `enum State` with a central `match` per operation. Rejected: every
new state would edit shared match sites — merge friction and a regression
surface on the existing states — working directly against additive growth
and the one-struct-per-file rule.

## Consequences
Each state co-locates its `(accept, step, cluster_view, admissible_commands)`
in one file. `step` returns `Box<dyn State>`, one allocation per transition —
acceptable, since transitions are rare relative to the work they gate.
Adding a state is adding a file plus its `(state × command)` matrix coverage.

## Enforcement
`core/src/states/` holds one file per state, each an `impl State`;
`core/src/state.rs` defines the trait. The transition matrix requires every
`(state, command)` pair — including every new state's — to be covered before
a phase is declared complete.
