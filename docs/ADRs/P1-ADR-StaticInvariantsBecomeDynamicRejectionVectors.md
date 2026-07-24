# P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors

- **Status:** Proposed
- **Date:** 2026-07-24
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

**Finding, not assumption:** a full read of `core/src` (all 25 files) and
`core/tests/property_tests/` (all three files) found no invariant that is
compile-time-only. Every transition that looks structurally prevented is
in fact a two-tier runtime contract in `Faction::process()`
(`core/src/faction.rs:42-83`): `Command::Probe` is intercepted first,
then `self.state.accept(&command)` is checked — only a command passing
both reaches `step()`. Every `unreachable!()` inside a `step()`
implementation is dead code given `process()`'s own calling order, not a
case the type system makes impossible to construct. Given this, the
decision is: state this as a checked, falsifiable claim in
`SPECIFICATION.md`'s "Structurally absent transitions" section — "no
statically-enforced invariant was identified as of `<date>`; every
invariant is enforced by the `Probe → accept() → step()` order in
`Faction::process()`" — rather than assume it silently or build
speculative `unconstructible_in` tooling for a category that may be
empty. If a future refactor introduces a genuine static invariant, the
corresponding vector is tagged `unconstructible_in: ["reference"]` at that
point, requiring a comment citing the exact enforcing mechanism —
evidence required at the time it's first used, not infrastructure built
ahead of need.

## Forcing constraints / Evidence

Every state (`Initial`, `Pinging`, `Collecting`, `Bootstrapped`) implements
`State` as a plain `Box<dyn State>` trait object; nothing restricts which
`Command` variant can be offered to which state's `step()`. The
`unreachable!()` messages themselves say so directly:
`"accept() rejects this command for Collecting"` (`states/collecting.rs:149`),
`"accept() rejects this command for Bootstrapped"` (`states/bootstrapped.rs:67`),
`"Probe handled in Faction::process"` (`states/pinging.rs:175`) — each
names a *calling-order* guarantee, not a type-level one.
`core/tests/property_tests/{invariant_tests,model_tests,safety_tests}.rs`
independently confirm this: every safety property there (exit is
terminal, counts never decrease, duplicate/non-member commands never
mutate state) is checked via `proptest` over generated runtime input,
never via a type constraint.

## Rejected alternatives

**Build the `unconstructible_in` evidence mechanism now, regardless of
whether any case currently needs it.** Rejected: costs real design effort
for a set that is empty today; the mechanism is trivial to add the moment
a genuine case appears, and building it speculatively risks it existing
unused and unreviewed for a phase or more.

**Skip stating the "zero static invariants" finding in the specification,
since there's nothing to enforce.** Rejected: an unstated assumption is
exactly what goes stale silently; a dated, falsifiable claim in
`SPECIFICATION.md` is what lets a future contributor re-check it rather
than inherit it as folklore.

## Consequences

Every current vector in Phase 1's scope is executable by every conformant
implementation, Rust or not — there is no vector the reference harness
gets to skip today. If a future phase (e.g. a typestate-based
reconfiguration guard in Phase 3 or 4) introduces a genuine static
invariant, that phase's own ADR states which enforcement class it falls
into, using this ADR's audit method: grep every call site of the type in
question; if the type system alone prevents an illegal value from ever
being constructed, it's static and needs an `unconstructible_in` vector;
otherwise it's dynamic and needs an ordinary rejection vector.

## Enforcement

None yet — enforced once the harness gate (reference harness executes
the full vector set, skips only where `unconstructible_in` permits) is
built. Until then, this ADR's finding is checked by re-running the same
audit method against current source, not by any automated gate.
