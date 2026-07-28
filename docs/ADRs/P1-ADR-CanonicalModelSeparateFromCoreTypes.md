# P1-ADR-CanonicalModelSeparateFromCoreTypes

- **Status:** Proposed
- **Date:** 2026-07-24
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

A separate, versioned, language-neutral canonical model is introduced,
with a `mapping.rs`-equivalent layer as the only code aware of both the
canonical vocabulary and `core`'s concrete types. Both response tiers are
mapped: `ProcessResult::Rejected` becomes its own canonical effect (e.g.
`rejected_not_admissible`, carrying `admissible`); each `Outcome`-level
soft-rejection (`JoinDenied`, `NonMemberIgnored`, `DuplicateMemberIgnored`,
`DuplicateParticipationIgnored`, `DuplicateReadyIgnored`) becomes its own
named canonical effect, alongside the "real" effects (`MemberAdmitted`,
`EmitJoinRequest`, `Concluded`, ...). No canonical effect is invented that
isn't already backed by an existing, tested Rust variant — the canonical
model maps what exists; it does not redesign it.

## Forcing constraints / Evidence

`core/src/process_result.rs` and `core/src/outcome.rs`, read directly:
`ProcessResult` has three variants, only one of which (`Accepted`) carries
an `Outcome` list; `Rejected` carries no reason beyond "not in `admissible`."
A canonical model that only translates `Outcome` cannot represent a
rejected command at all. Two rejection-shape alternatives were considered
before this decision (see Rejected alternatives) — mapping as-is was chosen
because every existing variant is already verified behavior, cited in
`core/tests/transition_matrix/` and `core/tests/property_tests/`.

## Rejected alternatives

**Unify all rejections into one `{"effect": "rejected", "reason": <enum>}`
shape**, matching the brief's illustrative JSON more literally. Rejected:
this requires inventing a `RejectReason` enum spanning both response tiers
that the Rust code does not currently express as one flat type, and
deciding whether `admissible` (today only present on `Rejected`/`Probed`)
becomes a universal field on every rejection — real design work trading
already-verified precision for a shape that only superficially resembles
the brief's example.

**Derive `Serialize` on `Command`/`Outcome`/`ProcessResult` directly and
export those.** Rejected per `P0-ADR-SpecificationIsNormativeCrateIsReference` —
this makes the crate's Rust types the de facto specification vocabulary,
the exact inversion that ADR exists to prevent.

## Consequences

The canonical model's effect vocabulary is wider than the brief's
illustrative example (9-10 named effects rather than one generic
rejection shape), but every one of them is traceable to an existing,
tested Rust variant with zero invented taxonomy. A future consumer adding
a new rejection kind adds both a new `Outcome`/`ProcessResult` variant and
its canonical counterpart in the same change — the mapping layer has
nowhere else for an untranslated variant to hide.

## Enforcement

None yet — enforced once the canonical-model crate/module and its mapping
layer exist and the completeness gate (every `Outcome`/`ProcessResult`
variant has a canonical counterpart) is wired into CI.
