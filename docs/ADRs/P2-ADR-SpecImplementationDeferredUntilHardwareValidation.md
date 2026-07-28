# P2-ADR-SpecImplementationDeferredUntilHardwareValidation

- **Status:** Proposed
- **Date:** 2026-07-24
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context

This cross-cutting specification effort was designed and decided (the six
sibling ADRs) before its own implementation begins, while two consumer
integrations of Phase 1 (dynamic joining) are still mid-flight:
`etheram-ibft-embassy` has stages 1-3 done, stages 4-6 (physical-hardware
validation) pending on hardware availability; `etheram-raft-embassy` has
the same stage 4-6 gate ahead of it; and `etheram-raft`'s own Phase 1
integration is not yet designed. Building the matrix refactor and
canonical model against Phase 1 now, before that hardware validation
completes, risks doing the mapping work against a target that real-hardware
testing could still change.

## Decision

The six sibling ADRs are recorded now, `Status: Proposed`. Implementation
of the cross-cutting specification effort (the matrix refactor, the
canonical-model crate, the vector exporter, the reference harness, the new
CI gates) does not begin until Phase 1 is validated on real NUCLEO
hardware across `etheram-ibft-embassy` and `etheram-raft-embassy` (stages
4-6 both), and `etheram-raft`'s own Phase 1 integration lands. That
hardware validation is also the gate for publishing this release of
`faction` to crates.io. The specification implementation follows
afterward, as its own delivery.

## Forcing constraints / Evidence

`etheram-ibft-embassy`'s and `etheram-raft-embassy`'s six-stage gates
(stages 4-6 requiring physical hardware, per each repo's own
`P2-ADR-SixStageGateHostThroughRealCluster.md`) are not yet green for
either repo as of this decision. `P2-ADR-TransitionMatrixSingleSourceForTestsAndVectors`'s
refactor and `P1-ADR-CanonicalModelSeparateFromCoreTypes`'s mapping layer
both operate directly on `Command`/`Outcome`/the transition matrix — any
change to that vocabulary forced by hardware findings would require
redoing both.

## Rejected alternatives

**Implement the specification effort immediately, in parallel with
hardware validation.** Rejected: if hardware testing surfaces a needed
change to the join logic itself, the matrix and canonical-model mapping
work done in parallel would need to be redone against the corrected
vocabulary — sequencing avoids paying that cost twice.

**Defer even deciding the specification questions until after hardware
validation.** Rejected: the design questions (canonical model shape,
matrix consolidation, static/dynamic invariant handling, ordering,
sequencing) do not depend on hardware behavior — they are answerable from
source today, and recording them now costs nothing while genuinely
unblocking review and planning in the meantime.

## Consequences

There is a real, visible gap between these ADRs' `Proposed` status and
their eventual implementation — this is deliberate, not an oversight, and
this ADR is the citable reason a reader should expect that gap rather than
assume the work stalled. The specification implementation, once it
begins, ships as its own delivery versioned to stay under `0.5.0`,
contingent on adding zero new `Command`/`Outcome`/`ProcessResult` variants
(a new enum variant is a breaking change for any downstream exhaustive
match) — purely additive alongside the already-published surface.

## Enforcement

None automated — enforced by this ADR being the recorded reason
implementation work on the cross-cutting specification effort should not
start before `ROADMAP.md`'s hardware-validation gate for Phase 1 is green
across both embedded consumers and `etheram-raft`'s own integration lands.
