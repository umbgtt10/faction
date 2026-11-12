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
compile-time-only. Every transition that looks structurally prevented is in fact
a two-tier runtime contract in `Faction::process()` (`core/src/faction.rs`):
`Command::Probe` is intercepted first, then `self.stage.accept(&command)` is
checked — only a command passing both reaches `step()`. Every `unreachable!()`
inside a `step()` implementation is dead code given `process()`'s own calling
order, not a case the type system makes impossible to construct. The
specification states it as a checked, falsifiable claim (`spec/SPEC.md` §5):
admission is decided when a command arrives, so every vector runs on every
conforming implementation, and none is tagged as unconstructible anywhere.

## Forcing constraints / Evidence

Every state (`Initial`, `Pinging`, `Collecting`, `Bootstrapped`) implements
`State`, and `Stage` delegates to it; nothing restricts which `Command` variant
can be offered to which state's `step()`. The `unreachable!()` messages say so
directly: `"accept() rejects this command for Collecting"`
(`states/collecting.rs`), `"accept() rejects this command for Bootstrapped"`
(`states/bootstrapped.rs`), `"Probe handled in Faction::process"`
(`states/pinging.rs`) — each names a *calling-order* guarantee, not a type-level
one. `ladder/tests/property/{invariant_tests,model_tests,safety_tests}.rs`
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
