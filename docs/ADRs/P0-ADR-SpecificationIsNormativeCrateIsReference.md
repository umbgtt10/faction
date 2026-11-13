# P0-ADR-SpecificationIsNormativeCrateIsReference

- **Status:** Accepted
- **Date:** 2026-11-12
- **Priority:** P0 (axiom)
- **Phase:** cross-cutting

## Context
`faction`'s consumers run on boards and on hosts, inside more than one protocol,
and may one day be written in more than one language. A definition that lives
only in Rust is one implementation's behaviour, its bugs included, and a second
implementation has nothing to be checked against. Every property claimed for the
machine — totality, determinism, protocol-agnosticism — was checkable only by
reading or testing the crate.

## Decision
`faction` is specified as a standard component, in a versioned `spec/` written by
hand — a JSON Schema data model, a JSON decision table, normative prose and JSON
conformance vectors — and this crate is one implementation of it: where the two
disagree, the specification is right and the crate has a bug.

## Forcing constraints / Evidence
It is practical here for the reasons it was for `beggar`: a pure machine with no
I/O and no clock (`P0-ADR-PureMealyNoIO`, `P0-ADR-DeterministicReplayable`) has no
encoding and no temporal logic to specify, and its one injected dependency, an
observer, is told and answers nothing, so an implementation runs the vectors with
nothing to stub.

The crate came first, so the specification is drawn from what it does — but not
from its tests. The prose and the decision table were written from the crate's
behaviour, every vector's expected answers by hand, and each vector was checked
against an independent model built from the prose before the crate was run on
it. The crate then answered all of them as written. That model is now in the
repository, `crates/spec-model/`, and checks every vector on every run
(`P1-ADR-IndependentModelOfTheSpecification`).

## Rejected alternatives
**The Rust crate as the definition**, its documentation and tests standing for a
specification. Rejected: tests in one language are not consumable by another,
and prose without vectors is not checkable.

**Vectors exported from the crate's tests.** Rejected: the crate becomes the
source the specification is drawn from, and a refactor redefines what conforming
means.

**`serde` derives on the crate's types, their JSON standing as the model.**
Rejected: it ties the format to one implementation's names and nesting, and
gives the crate a dependency.

**Vectors that state an implementation's representation** — the placeholder peer
id the crate gives a command it lists as admissible. Rejected: the specification
fixes behaviour, not representation, so a vector lists the admissible commands by
name.

**Numbers as JSON numbers.** Rejected: many JSON parsers read numbers as 64-bit
floats, exact only to 2⁵³, and a peer id is any u64.

## Consequences
The specification has four parts, versioned together:

| part | form | says |
|---|---|---|
| the data model | `schema.json`, a JSON Schema | every command, result, outcome and the view: fields and types |
| the matrix | `matrix.json`, a decision table | what each stage admits, and the rows in check order, each a named situation and its result |
| the semantics | `SPEC.md`, normative prose | what JSON cannot say: each situation's meaning, totality, determinism, the arrival order of peers |
| conformance | `vectors/*.json` | `(configuration, commands) → (results, final answer)`, each vector naming the rows it exercises |

Behaviour is in the specification before it is code: a change to the matrix is a
change to the specification first. A result carries the whole view, so any
implementation's log can be replayed by any tool.

The crate never reads the specification. Its conformance tests map the JSON to
its types through a model of their own. The specification and the crate are
versioned apart.

## Enforcement
The conformance tests run every vector; every vector validates against the
schema; every row of `matrix.json` is exercised by at least one vector. The
independent model answers every vector as written, each deciding by exactly the
rows it names, before the crate runs them.
