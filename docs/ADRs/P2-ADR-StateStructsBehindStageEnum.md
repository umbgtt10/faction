# P2-ADR-StateStructsBehindStageEnum

- **Status:** Accepted
- **Date:** 2026-11-12
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context
The machine must grow from its lifecycle stages to many more across Phases 2–6, without each addition destabilising the states that already exist.
It must also own its current state outright: the lifecycle is the machine's
internal business, never a dependency a consumer supplies.

## Decision
Each state is its own struct implementing one `State` trait — `step` and
`cluster_view`, neither with a default body — one struct per file under
`crates/core/src/states/`. `Faction` holds the current state as a `Stage`, an enum with
one variant per state struct (`crates/core/src/stage.rs`). Every command is offered to
`step`, which answers the outcomes and the next `Stage`, or nothing for a command
the stage does not admit — a probe among them — and `process` swaps the next
stage in. `Stage` implements `State` by delegating through one private `match`
that lends out the variant's struct as `&dyn State`, and derives
`admissible_commands()` from what `step` answers.

## Forcing constraints / Evidence
Two forces. Additive growth: a new state must not edit the code of the states
already there. And the house rule that a type never builds its own
dependencies — `cargo stern4rust`'s `injected-dependencies`: a field declared as
a trait receives only what the caller passed in. A `Box<dyn State>` field
breaks that rule twice: `Faction::new` would build the boxed `Initial` itself,
and every transition would replace the field with a box the caller never saw.
Injecting the initial state instead would hand every consumer the state graph
to assemble, against `P1-ADR-SingleEntryPoint`. A concrete `Stage` is owned state,
not a dependency, so the rule holds and the lifecycle stays inside the machine.

## Rejected alternatives
`Box<dyn State>` held by `Faction` — fails `injected-dependencies` as above, and
allocates once per transition. A single `enum State` whose variants carry the
states' fields and whose every operation `match`es over them — each new state
would edit every operation's match: merge friction and a regression surface on
the existing states. `Stage`'s one `match` maps a variant to its struct and does
nothing else; the per-state logic stays in the per-state files. Skipping
`injected-dependencies` for `faction` — a house rule that holds in every other
package, waived for the one whose structure is its product.

## Consequences
Adding a state is adding its file, one `Stage` variant and one arm in the
delegating `match`, plus its `(state × command)` matrix coverage; no existing
state's file changes unless that state transitions into the new one.
Transitions allocate no box. `admissible_commands()` cannot drift from what a
stage steps on: it is computed from `step` rather than declared beside it, at the
cost of stepping on each command to list them. No state holds an arm that cannot
be reached. A state
that hands over to another builds that `Stage` variant itself, so the states
know `Stage`'s variants — the price of returning owned data instead of a trait
object. What two states answer alike lives in one struct they share rather than
in a state of its own: `Reconfiguring` wraps `Bootstrapped`, and both answer
liveness through `Liveness` — `Probing` and `Suspicion` under `states/` — so its
rows are written once. A node's own removal, answered alike by every stage but
`Removed`, is each of those stages' to take, through `Removed::own_removal`:
`Stage` still only delegates.

## Enforcement
`crates/core/src/states/` holds one file per state, each an `impl State`;
`crates/core/src/state.rs` defines the trait with no default bodies; `crates/core/src/stage.rs`
holds the only `match` over the states. `crates/core/tests/stage_tests.rs` checks the
delegation for every variant, `validation/ladder/tests/matrix/` covers every
`(state, command)` pair, and `cargo stern4rust` enforces `injected-dependencies`
over `faction` in stage 2.
