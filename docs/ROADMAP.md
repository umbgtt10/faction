# `faction` — Roadmap

**Crate:** `faction`  
**License:** MIT  
**Date:** 2026-04-26  
**Status:** Phase 0 — Active  

---

## Vision

A protocol-agnostic, `no_std + alloc`, 0-unsafe, 100% static Mealy machine for distributed systems
bootstrapping, discovery, and dynamic membership. Published independently on crates.io.

**Core invariant:** `output = F(state, input)` — pure function, no side effects inside the machine.

---

## Design constraints (non-negotiable)

| Constraint | Rationale |
|---|---|
| `no_std + alloc` | Runs everywhere — NUCLEO, Jetson, cloud, WASM — no questions asked |
| 0 unsafe | Ownership-based correctness, passes procurement checklists |
| Pure Mealy machine | Trivially testable, deterministic replay, TLA+ mappable |
| Complete `(state, input)` coverage | Every pair explicitly tested — mandatory, not aspirational |
| Protocol-agnostic | No Raft, no IBFT, no Ethereum knowledge inside `faction` |
| Observer trait | Every transition pair reaches the observer — observability is not optional |
| Each phase is a strict superset | Phase N tests pass at Phase N+5. No exceptions. |

---

## Current state

`faction` (core) implements a **two-phase cluster readiness state machine** — a startup barrier
that coordinates when a group of nodes is ready to proceed.

**Phase 1 — Participation:** nodes observe participation signals from peers. Once a quorum of
participation observations is collected, the machine moves to Phase 2.

**Phase 2 — Readiness:** the node signals its own readiness. Remote readiness signals are
collected. Once a quorum of readiness signals is reached, the machine exits with `Quorum`.

**Deadline fallback:** if `DeadlineExpired` is triggered before quorum, the machine exits
with `Deadline`.

**Freshness classification:** each observation carries a freshness marker. The configurable
`FreshnessPolicy` classifies observations as `Timely`, `DelayedWithinMargin`, or `Stale`.

**`faction-validation`:** a deterministic scenario harness for multi-node readiness simulation.

**Property-based invariants verified by proptest:**
- Exit happens at most once
- Counts never decrease
- Stale/duplicate/non-member inputs never mutate state
- Deadline and quorum both lead to correct exited states

---

## Roadmap

### Phase 0 — Harden what exists
**Status:** Active  
**Target:** 1–2 weeks  

Before extending, make the existing machine bulletproof.

**Deliverables:**
- Complete `(state, input)` matrix coverage — every pair explicitly tested, not just property-based invariants
- Adversarial inputs — malformed freshness markers, duplicate signals, non-member signals, signals arriving after exit
- Observer trait coverage — every transition pair verified to reach the observer
- `faction-validation` harness extended to cover deadline races explicitly

**Gate:** 100% `(state, input)` coverage. Nothing advances until this is green.

---

### Phase 1 — Static membership registry
**Status:** Planned  
**Target:** 2–3 weeks  
**Depends on:** Phase 0 complete  

New question: **"Who is the cluster?"**

The startup barrier knows when quorum is reached but does not produce a durable membership set.
Phase 1 extends the machine to emit a `MembershipSnapshot` on exit.

**New states:**
- `Forming` — collecting participation/readiness signals
- `Formed(MembershipSnapshot)` — exited with quorum, membership known
- `Failed` — exited with deadline

**New inputs:**
- All existing inputs
- `QueryMembership` — caller asks for current confirmed set

**New outputs:**
- `MembershipSnapshot` — emitted on quorum exit, contains confirmed node set
- `MembershipResponse` — response to query

**Key design decision:** node identity and address are generic parameters. `faction` has no
opinion on what a node is or where it lives. The caller provides a type that implements
`NodeId` — a trait with minimal bounds (equality, ordering, no `std::net`).

**Gate:** complete `(state, input)` coverage before Phase 2.

---

### Phase 2 — Failure detection
**Status:** Planned  
**Target:** 3–4 weeks  
**Depends on:** Phase 1 complete  

New question: **"Is everyone still alive?"**

The membership set is now known and static. The machine tracks liveness without changing membership.
SWIM-style semantics: suspect → indirect probe → confirm or revive.

**New states:**
- `Stable` — membership known, all nodes suspected alive
- `Degraded(SuspectSet)` — one or more nodes suspected
- `QuorumLost` — suspicion has grown beyond threshold

**New inputs:**
- `ProbeAck { from: NodeId }` — successful probe response
- `ProbeMiss { from: NodeId }` — probe timeout
- `IndirectProbeAck { target: NodeId, via: NodeId }` — SWIM-style indirect confirmation
- `IndirectProbeMiss { target: NodeId, via: NodeId }`
- `Revive { node: NodeId }` — previously suspected node re-confirmed alive

**New outputs (commands to caller):**
- `SendProbe { to: NodeId }` — caller executes the probe
- `SendIndirectProbe { target: NodeId, via: NodeId }` — caller executes indirect probe
- `EmitQuorumLost` — caller's consensus layer is notified
- `EmitQuorumRestored` — recovery notification

**Design invariant:** the machine never touches the network. It emits commands, the caller
executes them, results come back as inputs. Pure Mealy throughout.

**Quorum threshold** is a caller-provided parameter — `faction` does not know what quorum
means, only whether the alive set satisfies the threshold the caller provided.

**Gate:** complete `(state, input)` coverage before Phase 3.

---

### Phase 3 — Single-node addition
**Status:** Planned  
**Target:** 2–3 weeks  
**Depends on:** Phase 2 complete  

New question: **"Can we grow by one?"**

The membership set is now mutable, but strictly one change at a time. The machine enforces
this structurally — it is impossible to request a second addition while one is pending.

**New states:**
- `Reconfiguring(PendingAddition)` — one addition in flight, all other change requests rejected
- Returns to `Stable` on commit or `Degraded` if the joining node is immediately suspected

**New inputs:**
- `RequestAdd { node: NodeId }` — caller requests addition
- `JoinAck { from: NodeId }` — joining node confirms it received the membership snapshot
- `AddCommitted` — caller's consensus layer confirmed the addition
- `AddAborted` — addition failed, rollback

**New outputs:**
- `SendMembershipSnapshot { to: NodeId }` — share current membership with joining node
- `RejectAdd { reason }` — addition rejected because one is already in flight
- `EmitMemberAdded { node: NodeId }` — committed, stable

**Core invariant:** at no point do two disjoint subsets of nodes each believe they form a
valid quorum. Single-at-a-time additions make this structurally impossible.

**Gate:** complete `(state, input)` coverage before Phase 4.

---

### Phase 4 — Single-node removal
**Status:** Planned  
**Target:** 2–3 weeks  
**Depends on:** Phase 3 complete  

New question: **"Can we shrink by one?"**

Symmetric to addition but with harder invariants. The machine refuses removals that would
break quorum — this is not a runtime check, it is a state transition that is simply
unavailable when the alive set would drop below threshold.

**New inputs:**
- `RequestRemove { node: NodeId }`
- `RemoveCommitted`
- `RemoveAborted`

**New outputs:**
- `RejectRemove { reason: RemoveRejectionReason }` — explicit reason: `WouldBreakQuorum`,
  `NodeAlreadySuspected`, `ReconfigurationInFlight`
- `EmitMemberRemoved { node: NodeId }`

**Degenerate cases — each is a first-class state, not an error path:**
- Removing a node already suspected dead — valid, handled differently from removing a live node
- Removing the last node in a minority partition — explicitly rejected
- Node removed while a probe is in flight — probe result arrives after removal, silently
  discarded via defined transition, not a state corruption

**Gate:** complete `(state, input)` coverage before Phase 5.

---

### Phase 5 — Full dynamic membership
**Status:** Planned  
**Target:** 4–6 weeks  
**Depends on:** Phase 4 complete  

New question: **"Can membership change arbitrarily under adversarial conditions?"**

The final phase removes the single-at-a-time constraint and introduces epochs, rejoin
handling, split-brain prevention, and concurrent change sequencing.

**Membership epochs:** every committed change increments a monotonic epoch counter. All
membership messages carry the epoch. Nodes with stale epochs produce a defined `StaleEpoch`
transition — not a crash, not a silent ignore.

**Rejoin handling:** a node that was previously removed can request to rejoin. The machine
distinguishes between a new node joining and a previously-known node rejoining — the state
transitions differ explicitly.

**Split-brain prevention:** the machine tracks the epoch at which each node last confirmed
membership. A node claiming quorum with an old epoch is explicitly rejected.

**Concurrent change sequencing:** multiple additions and removals can be requested but are
serialized by the machine into a bounded queue. The caller provides the bound.

**New states:**
- `SplitSuspected` — the machine suspects it may be in a minority partition
- `Rejoining(NodeId)` — a previously-removed node is being re-admitted

**Gate:** complete `(state, input)` coverage. crates.io publication readiness review.

---

## Publication readiness checklist

- [ ] All five phases complete with full `(state, input)` coverage
- [ ] `faction-validation` harness covers all phases adversarially
- [ ] `no_std + alloc` verified — no `std` dependency anywhere
- [ ] 0 unsafe — `#![deny(unsafe_code)]` at crate root
- [ ] Observer trait coverage — every transition pair verified
- [ ] README written as a standalone document — no EtheRAM references except in examples section
- [ ] At least one sample integration (Raft or IBFT) published as a companion crate
- [ ] crates.io metadata complete — description, keywords, categories, license
- [ ] CHANGELOG.md from Phase 0

---

## The `cartel` boundary

`faction` manages membership — who is in the cluster and whether they are alive. `faction`
emits `QuorumLost` and `QuorumRestored` events based on a threshold the caller provides,
but it does not compute quorum. That is `cartel`'s job.

The seam is explicit: `faction` → membership set + liveness events. `cartel` → quorum
computation over that set. The caller wires them together.

---

## Timeline summary

| Phase | Focus | Target duration |
|---|---|---|
| Phase 0 | Harden existing machine | 1–2 weeks |
| Phase 1 | Static membership registry | 2–3 weeks |
| Phase 2 | Failure detection (SWIM) | 3–4 weeks |
| Phase 3 | Single-node addition | 2–3 weeks |
| Phase 4 | Single-node removal | 2–3 weeks |
| Phase 5 | Full dynamic membership | 4–6 weeks |
| **Total** | | **14–21 weeks** |

---

## Hard rules

- Each phase produces a committable, independently useful artifact
- No phase begins until the previous phase has 100% `(state, input)` coverage
- Each phase is a strict superset of the previous — Phase N tests pass at Phase N+5
- No logic changes without a failing test first
- No unsafe code — ever
- The machine never performs I/O — it only emits commands
- Every degenerate case is a first-class state, not an error path
