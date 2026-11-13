# P1-ADR-CanonicalModelSeparateFromCoreTypes

- **Status:** Accepted
- **Date:** 2026-11-12
- **Priority:** P1 (derived from `P0-ADR-SpecificationIsNormativeCrateIsReference`)
- **Phase:** cross-cutting

## Context

`core`'s `Command` (8 variants) and `Outcome` (14 variants) are already
closed, domain-named enums — but they are Rust types, and a canonical,
language-neutral vocabulary for the specification must not be defined by
whatever a Rust refactor happens to name things. Separately, `process()`
returns two independent response tiers, not one: `ProcessResult`
(`Accepted` / `Rejected` / `Probed`, always returned) and the `Vec<Outcome>`
nested inside `Accepted` — a canonical model built from `Outcome` alone
would miss the `Rejected` tier entirely.

## Decision

The canonical model is the specification's data model, `spec/schema.json`:
versioned with the specification, language-neutral, and defined by it rather
than by the crate. It keeps the vocabulary's names, and it carries both response
tiers: a result is `Accepted`, `Rejected` or `Probed`, a rejection carrying the
admissible commands; and every `Outcome` — the soft rejections `JoinDenied`,
`NonMemberIgnored`, `DuplicateMemberIgnored`, `DuplicateParticipationIgnored` and
`DuplicateReadyIgnored` among them — is a value of its own, beside the effects
(`MemberAdmitted`, `EmitJoinRequest`, `Concluded`, ...). No value is invented that
an existing, tested Rust variant does not back: the model maps what exists. The
only code aware of both the model and the crate's types is the conformance
tests' mapping.

## Forcing constraints / Evidence

`crates/core/src/process_result.rs` and `crates/core/src/outcome.rs`, read directly:
`ProcessResult` has three variants, only one of which (`Accepted`) carries an
`Outcome` list; `Rejected` carries no reason beyond the admissible commands. A
model that only translated `Outcome` could not represent a rejected command at
all. Every variant is already verified behaviour, exercised by the transition
matrix and the property tests.

## Rejected alternatives

**Unify all rejections into one `{"rejected": <reason>}` shape.** Rejected: it
needs a reason enum spanning both response tiers that the crate does not
express as one flat type, and it trades already-verified precision for a shape
that only resembles one.

**Derive `Serialize` on `Command`/`Outcome`/`ProcessResult` and export those.**
Rejected per `P0-ADR-SpecificationIsNormativeCrateIsReference` — the crate's Rust
types would become the specification's vocabulary.

**A canonical-model crate with a mapping layer of its own.** Rejected: the crate
would depend on, or be depended on by, a model of itself. The mapping is the
conformance tests' business, and the crate stays unaware of the specification.

## Consequences

A new command or outcome arrives in the schema first, then in the crate, then in
the tests' mapping, in the same change. A variant the mapping does not know does
not compile: the mapping matches every variant.

## Enforcement

Every vector validates against `schema.json`; the conformance tests map every
`Command`, `Outcome` and `ProcessResult` variant through exhaustive matches, and
compare each result as the schema writes it.
