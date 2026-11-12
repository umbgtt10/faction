# P2-ADR-SpecifiedBeforeAnyNewPhase

- **Status:** Accepted
- **Date:** 2026-11-12
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context
The specification was decided in July and deferred until Phase 1 had been
validated on hardware by its embedded consumers. Faction's next phases — member
addition, failure detection, removal — each add commands and outcomes, and Raft's
membership work waits on them.

## Decision
`faction` is specified now, at its first phase's vocabulary, before any later
phase adds to it; from here every phase changes the specification first, then
the crate.

## Forcing constraints / Evidence
A specification retrofitted after Phases 2 to 4 would have to recover three
phases' behaviour from code — the cost the deferral meant to avoid, multiplied.
Specified now, each phase extends a specification instead. The vocabulary the
deferral protected has not moved since Phase 1, and `beggar`'s specification
showed the method on the sibling crate.

## Rejected alternatives
**Wait for Phase 1's hardware validation**, as decided in July. Rejected: the
phases that follow would be built unspecified in the meantime, and a change
hardware forces is a change to the specification like any other.

## Consequences
The specification's 0.1.0 is Phase 1's vocabulary: bootstrapping and joining.
The crate's next release, 0.5.0, is the first to conform to it. Each later phase
opens with its specification's next version.

## Enforcement
A phase's plan begins with its change to `spec/`; the conformance tests run
every vector in stage 1.
