# Architecture Decision Records

Each ADR documents one load-bearing property of `faction` — succinct,
self-contained, citable on its own. Priority marks the dependency tier, not
importance:

- **P0** — foundational axiom; the machine's identity rests on it.
- **P1** — derived or shape decision; follows from a P0.
- **P2** — structural; real, but below the core. (Two further P2 items remain deferred in `../OPEN_POINTS.md` §10.)

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
| [P2-ADR-TotalObservability](P2-ADR-TotalObservability.md) | Every transition, query, and rejection reaches the `Observer`, on every path. |
| [P2-ADR-StateAsTraitObject](P2-ADR-StateAsTraitObject.md) | One struct per state behind a `State` trait; growth is additive. |

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
