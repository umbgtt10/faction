# P1-ADR-IndependentModelOfTheSpecification

- **Status:** Accepted
- **Date:** 2026-11-12
- **Priority:** P1 (derived from `P0-ADR-SpecificationIsNormativeCrateIsReference`)
- **Phase:** cross-cutting

## Context
Every vector's expected answers are written by hand, and a hand-written answer
can be wrong. Before the crate ran them, the vectors of specification 0.1.0 were
checked against an independent model built from the prose — a throwaway script
outside the repository, so the check could not be made again. Every later phase
changes the specification first (`P2-ADR-SpecifiedBeforeAnyNewPhase`), and each
change brings vectors that need a judge other than the crate: the crate as the
judge is the alternative `P0-ADR-SpecificationIsNormativeCrateIsReference`
rejects.

## Decision
The specification has a second implementation in the repository: the package
`faction-spec-model` in `spec-model/`, written from the prose and the decision
table and independent of the crate, `no_std` and free of unsafe code, gated like
the library. The vectors are checked against it before the crate runs them, and
the crate is checked against it over command sequences no vector holds.

## Forcing constraints / Evidence
**Independent of the crate, or worthless.** A model that shares the crate's types
or its shape shares its mistakes. So the model depends on nothing of `faction` —
`cargo deny` refuses `faction` to every crate but those written for it, and was
seen to refuse it to the model, as a dependency and as a dev-dependency — and
takes and answers commands in the specification's own encoding. The two meet
only in the ladder, where the crate's answers are written in that encoding.

**The specification's shape, not the crate's.** A command is answered by the
first row of the decision table that applies, in the table's order (§3). The
model reads the rows from `spec/matrix.json` and owns only what the prose gives
each: the situation in which it applies (eight of them, from "always" to "with
`peer_id` the last collecting peer, the collecting peers number the required
count or more"), the change it makes (six), the peers its outcomes name, and the
view (§4). The crate is states behind a stage enum
(`P2-ADR-StateStructsBehindStageEnum`); the model is a table read and
interpreted. Records hold the data — the command, the configuration, the node's
state, a row, a rule — and objects the behaviour: the reader of the encoding,
the table, the rule book, the view.

**What it found.** All 41 vectors, 103 steps and their final answers, answered
as written; each vector decided by exactly the rows it names; every row of the
matrix decided by some vector; the model's rule book holding exactly the
matrix's rows; and the matrix admitting in each stage what the prose's §5
states. Then, over random sequences of up to 40 commands — every command, its
peer among a five-member cluster's ids and two that are not members, every
required count from none to more than the cluster holds — the crate answering
every command as the model does.

## Rejected alternatives
**The script, kept outside the repository.** Rejected: a check no one can
repeat, held to no gate, is evidence once.

**The crate as the judge of its own vectors.** Rejected by
`P0-ADR-SpecificationIsNormativeCrateIsReference`: a vector the crate answers is
checked against nothing but the crate.

**A model built on the crate's types.** Rejected: one mistake in a shared type is
in both, and the two agree on it.

**The decision table typed into the model.** Rejected: the table is normative
data. Read, it is the table the specification states; the model's own knowledge
is what the table cannot say, and its rule book is checked to know exactly the
table's rows.

## Consequences
The model is independent of the crate's code and types, not of its author: a
misreading of the prose by one hand can be made twice. The vectors are the
check on both.

A change to the specification changes the model with it. A row the matrix gains
needs its situation and its change in the rule book before the conformance tier
passes; a command, an outcome or a stage the vocabulary gains is named there;
and the new vectors run against the model before the crate runs them.

The model is one node and has no observer: what the observer is told is the
crate's to show, in its own conformance tier. It reads the matrix, so it cannot
disagree with it about a row's result, outcomes or next stage — only about the
situations, the changes, the peers and the view; the matrix itself is checked
against the vectors and against the prose's admission.

The property tier's own reference model predates the specification and is now
redundant beside this one; retiring it is a decision of its own. The system
tests' logs can be replayed against the model (`docs/OPEN_POINTS.md`).

## Enforcement
`cargo deny`'s bans in `deny.toml`. The model's conformance tier,
`ladder/tests/conformance/spec_model_tests.rs`, and the crate against the
model, `ladder/tests/property/spec_model_tests.rs`. Stage 2 holds the model to
every gate the library is held to — stern4rust, dry4rust, crap4rust, twin4rust,
iceberg4rust, and coverage over every tier and over the conformance tier alone
(`P1-ADR-NoLineTheSpecificationDoesNotAskFor`); stage 1 builds it for a target
with no `std`; `#![forbid(unsafe_code)]`; and every commit message carries its
grip and braintax.
