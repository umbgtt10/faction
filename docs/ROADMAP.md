# `faction` — Roadmap

**Crate:** `faction`  
**License:** MIT  
**Date:** 2026-05-01  
**Status:** Phase 0 — Complete  

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

## Phase 0 — Static membership (cluster readiness)

**Status:** Complete  
**Lines of code (productive):** 1,165  
**Tests:** 275 total  
**Crappy functions:** 0  
**Code coverage (productive):** 100%

Full specification, state machine description, and limitations
in [PHASE-0-SPECIFICATION.md](./PHASE-0-SPECIFICATION.md).

---

## Roadmap

### Phase 1 — Dynamic membership: joining
**Status:** Planned  
**Target:** 2–3 weeks  
**Removes:** L1 (static membership)  

New question: **"Can a new peer join the cluster?"**

Phase 1 addresses the first and most fundamental limitation: the peer set is no longer
immutable. A new peer can send a join signal and be admitted to the cluster's member set
at runtime.

**Minimal definition of "joining":**
- A peer sends a `Join { peer_id }` command
- The machine either admits the peer (adds them to the member set) or rejects them
- Once admitted, the peer's signals (participation, readiness) are treated as valid member signals
- A non-admitted peer's signals continue to be ignored
- No reconfiguration protocol, no epochs, no failure detection

**New constraints that emerge:**
- `is_member` can no longer be computed from a static config — it must be tracked as part of state
- The `Config` type's `.is_member()` method becomes obsolete as a static query
- The non-member gate (`non_member_peer`) transitions from returning `NonMemberIgnored` to potentially
  producing `JoinOffer` or similar

**Key design question — what happens when a non-member sends a signal?**
- Option A: signal is ignored as before (current behavior, no change)
- Option B: signal triggers a `JoinRequested` output — the caller decides whether to admit
- Option C: signal is treated as an implicit join request — the machine admits automatically

**How this differs from Phase 3:** Phase 1 is a lightweight admission gate —
a peer either joins or doesn't. There is no commit/abort protocol, no membership
snapshot exchange, and no in-flight reconfiguration state. Phase 3 adds proper
reconfiguration with atomic commit and structural single-change enforcement.

**Gate:** complete `(state, input)` coverage before Phase 2.

---

### Phase 2 — Failure detection
**Status:** Planned  
**Target:** 3–4 weeks  
**Depends on:** Phase 1 complete  

New question: **"Is everyone still alive?"**

The membership set is now known and mutable. The machine tracks liveness without changing membership.
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
- [ ] CHANGELOG.md from Phase 0 through publication

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
| Phase 0 | Static membership (cluster bootstrapping) | Complete |
| Phase 1 | Dynamic membership: joining | 2–3 weeks |
| Phase 2 | Failure detection (SWIM) | 3–4 weeks |
| Phase 3 | Single-node addition | 2–3 weeks |
| Phase 4 | Single-node removal | 2–3 weeks |
| Phase 5 | Full dynamic membership | 4–6 weeks |
| **Total** | | **13–20 weeks** |

---

## Hard rules

- Each phase produces a committable, independently useful artifact
- No phase begins until the previous phase has 100% `(state, input)` coverage
- Each phase is a strict superset of the previous — Phase N tests pass at Phase N+5
- No logic changes without a failing test first
- No unsafe code — ever
- The machine never performs I/O — it only emits commands
- Every degenerate case is a first-class state, not an error path
