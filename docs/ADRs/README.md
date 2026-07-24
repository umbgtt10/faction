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
| [P2-ADR-TotalObservability](P2-ADR-TotalObservability.md) | Every transition, query, and rejection reaches the `Observer`, on every path. |
| [P2-ADR-StateAsTraitObject](P2-ADR-StateAsTraitObject.md) | One struct per state behind a `State` trait; growth is additive. |
| [P2-ADR-ClusterViewBuilderAndDto](P2-ADR-ClusterViewBuilderAndDto.md) | The view consumers receive is a pure DTO; construction lives in a separate builder. |
| [P2-ADR-TestingLadder](P2-ADR-TestingLadder.md) | A fixed test-tier ladder — unit, matrix, property, simulation, system — run by one gate. |
| [P0-ADR-SpecificationIsNormativeCrateIsReference](P0-ADR-SpecificationIsNormativeCrateIsReference.md) | From spec v1.0.0, the specification is normative and this crate is a conformant reference implementation. **Proposed** — decided, not yet built. |
| [P1-ADR-CanonicalModelSeparateFromCoreTypes](P1-ADR-CanonicalModelSeparateFromCoreTypes.md) | A language-neutral canonical model, mapped from (not derived-`Serialize`-on) `Command`/`Outcome`/`ProcessResult`. **Proposed**. |
| [P1-ADR-TransitionMatrixSingleSourceForTestsAndVectors](P1-ADR-TransitionMatrixSingleSourceForTestsAndVectors.md) | The six existing test files become one declarative `matrix.rs`, consumed by both the test suite and the vector exporter. **Proposed**. |
| [P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors](P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors.md) | No compile-time-only invariant was found in `core/src`; every invariant is the `Probe → accept() → step()` runtime contract. **Proposed**. |
| [P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder](P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder.md) | Effects are an ordered sequence; collections serialize in insertion order — both already true, not new behavior. **Proposed**. |
| [P2-ADR-CanonicalModelBuiltAgainstJoiningBranch](P2-ADR-CanonicalModelBuiltAgainstJoiningBranch.md) | The canonical model targets `Command`/`Outcome` with Phase 1 joining already included — there is no smaller baseline left. **Proposed**. |
| [P2-ADR-SpecImplementationDeferredUntilHardwareValidation](P2-ADR-SpecImplementationDeferredUntilHardwareValidation.md) | Decided now; implementation waits for Phase 1's NUCLEO hardware validation across `ibft-embassy`/`raft-embassy`/`raft`. **Proposed**. |

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
