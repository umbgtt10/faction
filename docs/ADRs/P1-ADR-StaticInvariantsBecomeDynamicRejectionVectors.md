# P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors

- **Status:** Accepted
- **Date:** 2026-11-12
- **Priority:** P1 (derived from `P0-ADR-SpecificationIsNormativeCrateIsReference`)
- **Phase:** cross-cutting

## Context

A specification whose safety argument rests on invariants Rust's type
system enforces at compile time cannot be conformance-checked by a
language without that type system — a Go port can only reject an illegal
transition at runtime. Before deciding how to handle that gap, it has to
be known which invariants (if any) actually are compile-time-only in this
codebase.

## Decision

**Finding, not assumption:** a full read of `core/src` and
`ladder/tests/property/` (all three files) found no invariant that is
compile-time-only. Every transition that looks structurally prevented is decided
at run time by the stage's `step`, which every command reaches: it answers nothing
for a command its stage does not admit, and `Faction::process` answers that
command `Rejected` — or `Probed`, for a probe. No `step` holds an arm that cannot
be reached. The specification states it as a checked, falsifiable claim
(`spec/SPEC.md` §5): admission is decided when a command arrives, so every vector
runs on every conforming implementation, and none is tagged as unconstructible
anywhere.

## Forcing constraints / Evidence

Every state (`Initial`, `Pinging`, `Collecting`, `Bootstrapped`) implements
`State`, and `Stage` delegates to it; every `Command` variant is offered to every
state's `step()`, which ends in an arm for the commands its stage does not admit.
`ladder/tests/property/{invariant_tests,safety_tests,spec_model_tests}.rs`
confirm it: every safety property there is checked by `proptest` over generated
runtime input, never by a type constraint.

## Rejected alternatives

**An `unconstructible_in` mechanism, built before any case needs it.** Rejected:
real design effort for a set that is empty; it is trivial to add the moment a
genuine case appears.

**Leaving the finding unstated, since there is nothing to enforce.** Rejected: an
unstated assumption is exactly what goes stale silently; a claim in the
specification lets a future contributor re-check it.

## Consequences

Every vector is executable by every conformant implementation, Rust or not. If a
future phase introduces a genuine static invariant — a typestate guard, say —
that phase's ADR states which class it falls into, by this ADR's audit method:
grep every call site of the type; if the type system alone prevents an illegal
value from being constructed, it is static and its vector says so; otherwise it
is dynamic and its rejection is an ordinary vector.

## Enforcement

The conformance tests run every vector, with no skip mechanism to run any fewer.
