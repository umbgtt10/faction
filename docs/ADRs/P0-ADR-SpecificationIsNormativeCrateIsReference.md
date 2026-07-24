# P0-ADR-SpecificationIsNormativeCrateIsReference

- **Status:** Proposed
- **Date:** 2026-07-24
- **Priority:** P0 (axiom)
- **Phase:** cross-cutting

## Context

`faction` has one implementation, in Rust, and no independent specification.
Every property claimed for the machine — purity, exhaustive transition
coverage, protocol-agnosticism — is currently checkable only by reading or
testing the Rust crate. A second, non-Rust implementation has no artifact to
conform to except the crate's own source, which makes "conforms to faction"
unverifiable for anyone who isn't reading Rust.

## Decision

From spec v1.0.0, the specification is normative and this crate is a
conformant reference implementation. Where the two disagree, the
specification is correct and the crate has a bug. Design questions are
settled in the specification first, then implemented; if implementation
reveals the spec is wrong, the spec is amended deliberately, with the
change recorded, not silently reconciled by changing the code to match a
belief about what the spec "must have meant."

## Forcing constraints / Evidence

No `spec/`, `vectors/`, or canonical-model directory exists anywhere in this
repository today — confirmed by direct search. Every current guarantee
(exhaustive `(state, command)` coverage, purity, zero unsafe) is stated in
`ARCHITECTURE.md` and enforced by Rust-side tests only. Nothing today
prevents a Rust-internal refactor from silently changing what "conforms to
faction" means, because nothing external to the crate currently defines it.

## Rejected alternatives

**Keep the crate as the sole source of truth; document behavior informally
in `ARCHITECTURE.md`.** Rejected: this is the status quo, and it is exactly
what makes a non-Rust conformant implementation unverifiable — a written
description is not a machine-checkable conformance target.

**Let the Rust types double as the specification (derive `Serialize` on
core types, ship those as the vectors).** Rejected — this makes the crate
normative by default despite this ADR's decision, and ties the vector
format to Rust implementation details (enum variant names, struct nesting)
that have no reason to survive a refactor.

## Consequences

Every subsequent design question in this cross-cutting initiative is
answered in specification terms first: a canonical, language-neutral model
(separate ADR), a closed effect vocabulary, reachable-by-construction
vectors, and CI gates that fail if the crate and the exported vectors
drift apart. Until those exist, this ADR states an intent this repository
does not yet enforce — it is `Proposed`, not `Accepted`, until the drift,
completeness, harness, and format-stability gates it depends on are built
and green.

## Enforcement

None yet — this is the founding decision the rest of the cross-cutting
ADRs build enforcement for. It becomes enforced once the drift gate
(regenerate vectors, byte-compare against committed output) exists and
runs in CI.
