# Architecture Decision Records

Each ADR documents one load-bearing property of `faction` — succinct,
self-contained, citable on its own. Priority marks the dependency tier, not
importance:

- **P0** — foundational axiom; the machine's identity rests on it.
- **P1** — derived or shape decision; follows from a P0.
- **P2** — structural; real, but below the core. (Further deferred/unwritten ADRs are tracked in `../OPEN_POINTS.md`.)

## Index

| ADR | Property |
|---|---|
| [P0-ADR-ProtocolAgnostic](P0-ADR-ProtocolAgnostic.md) | Zero consensus knowledge; the fault model is injected, never computed. |
| [P0-ADR-PureMealyNoIO](P0-ADR-PureMealyNoIO.md) | Emits outcomes, never executes; no I/O, no side effects. |
| [P0-ADR-DeterministicReplayable](P0-ADR-DeterministicReplayable.md) | No clock, RNG, or ambient input; `output = F(state, input)`. |
| [P0-ADR-NoStdZeroUnsafe](P0-ADR-NoStdZeroUnsafe.md) | `no_std + alloc`, zero unsafe; one binary, bare-metal to cloud. |
| [P1-ADR-SingleEntryPoint](P1-ADR-SingleEntryPoint.md) | One self-describing `process(command)` over a tri-variant result. |
| [P1-ADR-TotalityExhaustiveMatrix](P1-ADR-TotalityExhaustiveMatrix.md) | Every `(state, command)` is a defined transition; the matrix proves it. |
| [P1-ADR-StatefulPersistencyFree](P1-ADR-StatefulPersistencyFree.md) | Stateful, but persists nothing; durability is the consumer's. |
| [P1-ADR-TerminalStatesAreNotSinks](P1-ADR-TerminalStatesAreNotSinks.md) | A concluded node re-advertises to still-trying peers; a missed deadline is non-terminal. |
| [P1-ADR-ConfigIsImmutableGenesis](P1-ADR-ConfigIsImmutableGenesis.md) | `Config` is the immutable genesis seed; live membership is state, grown only by a command. |
| [P1-ADR-CollectingIsNotASink](P1-ADR-CollectingIsNotASink.md) | `Collecting` re-advertises readiness to a member's ping, like `Bootstrapped`; no silent sinks after local completion. |
| [P2-ADR-TotalObservability](P2-ADR-TotalObservability.md) | Every call reaches the `Observer` once, with the command and its result, on every path. |
| [P2-ADR-StateStructsBehindStageEnum](P2-ADR-StateStructsBehindStageEnum.md) | One struct per state behind a `State` trait, held in a `Stage` enum that only delegates; growth is additive. |
| [P2-ADR-StateAsTraitObject](P2-ADR-StateAsTraitObject.md) | The states held as `Box<dyn State>`. **Superseded** by `P2-ADR-StateStructsBehindStageEnum`. |
| [P1-ADR-AdditionThroughTheLayerThatCommits](P1-ADR-AdditionThroughTheLayerThatCommits.md) | A concluded node grows by one through the layer that commits changes above it: one addition in flight, the commit applied on every node, the count changed by its own command. |
| [P1-ADR-LivenessReportedNotStaged](P1-ADR-LivenessReportedNotStaged.md) | A concluded node runs SWIM's failure detector, the caller keeping its clock and its wire: liveness is reported in the view, never a stage; each node judges for itself; a suspect stays a member. |
| [P2-ADR-ClusterViewBuilderAndDto](P2-ADR-ClusterViewBuilderAndDto.md) | The view consumers receive is an immutable read-model; construction lives in a separate builder. |
| [P2-ADR-TestingLadder](P2-ADR-TestingLadder.md) | A fixed test-tier ladder — unit, matrix, property, simulation, system — run by one gate. |
| [P0-ADR-SpecificationIsNormativeCrateIsReference](P0-ADR-SpecificationIsNormativeCrateIsReference.md) | The specification in `spec/` — schema, decision table, prose, vectors, written by hand — is normative; this crate is one implementation of it. |
| [P1-ADR-NoLineTheSpecificationDoesNotAskFor](P1-ADR-NoLineTheSpecificationDoesNotAskFor.md) | 100% coverage over every tier and over the conformance vectors alone: no line the specification does not ask for. |
| [P1-ADR-IndependentModelOfTheSpecification](P1-ADR-IndependentModelOfTheSpecification.md) | A second implementation of the specification, `crates/spec-model/`, independent of the crate: it checks the vectors before the crate runs them, and the crate over sequences no vector holds. |
| [P1-ADR-CanonicalModelSeparateFromCoreTypes](P1-ADR-CanonicalModelSeparateFromCoreTypes.md) | The canonical model is `spec/schema.json`, mapped to `Command`/`Outcome`/`ProcessResult` by the conformance tests, never derived from them. |
| [P1-ADR-TransitionMatrixSingleSourceForTestsAndVectors](P1-ADR-TransitionMatrixSingleSourceForTestsAndVectors.md) | A `matrix.rs` exported to vectors. **Superseded** by `P0-ADR-SpecificationIsNormativeCrateIsReference`: the decision table is `spec/matrix.json`. |
| [P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors](P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors.md) | No compile-time-only invariant exists in `crates/core/src`; admission is decided at run time, so every vector runs everywhere. |
| [P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder](P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder.md) | Outcomes are an ordered sequence; peers are listed in arrival order. |
| [P2-ADR-CanonicalModelBuiltAgainstJoiningBranch](P2-ADR-CanonicalModelBuiltAgainstJoiningBranch.md) | The specification's 0.1.0 includes joining — there is no smaller baseline left. |
| [P2-ADR-SpecifiedBeforeAnyNewPhase](P2-ADR-SpecifiedBeforeAnyNewPhase.md) | Faction is specified now, at Phase 1's vocabulary; every later phase changes the specification first. |
| [P2-ADR-SpecImplementationDeferredUntilHardwareValidation](P2-ADR-SpecImplementationDeferredUntilHardwareValidation.md) | The specification deferred until hardware validation. **Superseded** by `P2-ADR-SpecifiedBeforeAnyNewPhase`. |
| [P2-ADR-WorkspaceLaidOutByRole](P2-ADR-WorkspaceLaidOutByRole.md) | The workspace by role: `spec/` the specification, `crates/` the library, `validation/` what checks it, `xtask/` the gates. |

## Template

```markdown
# <Priority>-ADR-<Name>

- **Status:** Accepted | Proposed | Superseded by <ADR>
- **Date:** YYYY-MM-DD
- **Priority:** P0 (axiom) · P1 (derived) · P2 (structural)
- **Phase:** 0 · 0-bugfix · 1–6 · cross-cutting

## Context
The forces and tension this resolves.

## Decision
The choice, in one quotable sentence.

## Forcing constraints / Evidence
Why this was forced, not freely chosen — the real evidence. `N/A` if none.

## Rejected alternatives
What we did not do, and why.

## Consequences
What it commits us to; what it costs; obligations pushed onto consumers.

## Enforcement
The specific test, gate, or structural mechanism that keeps it true.
```

Fields that do not apply are marked `N/A` rather than padded.

Each ADR is a snapshot of the decision as it stands today, not a changelog:
state the current shape as fact, and do not narrate what an earlier version
of this document — or of any other document — used to say. Keep
cross-references minimal; link another ADR only where the relationship is
load-bearing, and never cite a living or frequently-changing document
(`OPEN_POINTS.md`, `CHANGELOG.md`, or similar tracking/status files) — an
ADR whose meaning depends on another document's current state goes stale
the moment that document changes.
