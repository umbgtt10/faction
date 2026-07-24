# P1-ADR-TransitionMatrixSingleSourceForTestsAndVectors

- **Status:** Proposed
- **Date:** 2026-07-24
- **Priority:** P1 (derived from `P0-ADR-SpecificationIsNormativeCrateIsReference`)
- **Phase:** cross-cutting

## Context

"The matrix" is not one artifact today. It is six files in two idioms:
`state_transition_matrix_tests.rs` (the accepted-command path, `rstest`
`#[case::name(...)]` macros), four per-state `*_invalid_tests.rs` files
(the rejected-command path, same macro idiom), and
`admissible_invariant_tests.rs` (a property cross-check over both). None of
this is freestanding data — every case lives inside a macro invocation, so
nothing outside `cargo test` can read "the matrix" today.

## Decision

All six files' cases are extracted into one declarative `matrix.rs` table,
with rows for both response shapes (`Accepted { outcomes, ... }` /
`Rejected { admissible }`), consumed by both the existing test suite and a
new vector-exporter tool. Adding a transition means adding a row; both
consumers update automatically. `admissible_invariant_tests.rs` stays a
separate, ordinary property test over the matrix — it is a cross-check,
not matrix data itself.

## Forcing constraints / Evidence

Confirmed directly by reading all six files: `state_transition_matrix_tests.rs`
and the four `*_invalid_tests.rs` files use identical `#[rstest]`
`#[case::name(...)]` structure; nothing reads their case data except
`cargo test` itself. A vector-exporter with no separate data source to
read from would have to either re-derive cases by parsing test macros (
fragile) or duplicate them by hand (drifts immediately) — neither
satisfies `P0-ADR-SpecificationIsNormativeCrateIsReference`'s requirement
that the spec and the crate cannot silently diverge.

## Rejected alternatives

**Keep the six files as-is; hand-write a parallel vector set.** Rejected:
this is exactly the duplication `P0-ADR-SpecificationIsNormativeCrateIsReference`
exists to prevent — two sources of truth for the same transitions, with no
structural mechanism keeping them in sync.

**Parse the existing `rstest` macro invocations at build time to generate
vectors.** Rejected: fragile (couples the exporter to `rstest`'s macro
syntax rather than to a stable data shape) and harder to review than a
plain data table.

## Consequences

This is the largest mechanical refactor in the whole cross-cutting
initiative — six files' worth of existing, working test cases move into a
new shape, executed as its own additive-first sub-wave (add `matrix.rs`
alongside the current tests unchanged; migrate tests to consume it,
confirming identical pass/fail; only then build the exporter against it),
gate green after each step. The property test is unaffected and needs no
migration.

## Enforcement

None yet — enforced once `matrix.rs` exists and both `cargo test` and
`tools/vector-export` demonstrably read from it, backed by the
completeness gate (every matrix row appears in at least one exported
vector).
