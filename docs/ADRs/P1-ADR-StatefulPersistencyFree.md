# P1-ADR-StatefulPersistencyFree

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P1 (derived)
- **Phase:** cross-cutting

## Context
Across Phases 1–6 the machine's state grows — member set, in-flight-change
guard, epoch counter, and more. Something must own that state durably across a
restart; the question is whether it is `faction` or the consumer.

## Decision
`faction` persists nothing, ever. It is stateful, but it never durably owns its
state: state lives in memory and is reconstructed by deterministic replay of
the input log. Durability is entirely the consumer's.

## Forcing constraints / Evidence
Entailed by the machine's purity and determinism: persistence is I/O (which
the machine does not do), and replay only reconstructs state faithfully
because the machine is deterministic. Hardware evidence: in a crash-recovery
run a rebooted node came up at height zero and rebuilt its entire state by
replaying peers' committed history — the same pattern applies to membership.

## Rejected alternatives
A `faction`-owned durable membership store. Rejected: it creates a second
source of truth that must hand-mirror the consumer log's
roll-back-on-truncation semantics, and it performs the I/O the machine forbids.

## Consequences
On restart the consumer replays committed change history through `faction` to
rebuild state — unbounded for a long-lived cluster. The consumer bounds this by
snapshotting the derived view (members, epoch, log index) and replaying only
the tail. `faction` owns the fold; the consumer owns the disk.

## Enforcement
No durable handle exists anywhere in the crate; replay-equivalence (same input
sequence → same state) is covered by the transition matrix.
