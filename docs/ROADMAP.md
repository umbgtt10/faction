# Roadmap

**Crate:** `faction`  
**License:** MIT  
**Last updated:** 2026-07-17  
**Current status:** Phase 0 — Complete

> **Note:** All inputs, outputs, and state names listed for Phases 1–6 below
> are drafts. They reflect the intended design direction but may change during
> implementation.

---

## Vision

A protocol-agnostic, `no_std + alloc`, 0-unsafe Mealy state machine that solves the
full node lifecycle management problem for distributed systems — once, correctly, and
for every protocol that needs it.

**The core invariant never changes:** `output = F(state, input)`. Pure function. No
side effects inside the machine. Every phase extends the input and state spaces without
violating this invariant.

---

## Non-negotiable constraints

These constraints apply to every phase. They are not relaxed as the machine grows more
complex.

| Constraint | Rationale |
|---|---|
| `no_std + alloc` | Runs on bare metal, WASM, embedded RTOS, and cloud — the same binary |
| 0 unsafe | Ownership-based correctness. Passes security procurement checklists. |
| Pure Mealy | `output = F(state, input)`. Trivially testable. Deterministically replayable. |
| Complete `(state, input)` coverage | Every pair explicitly tested — mandatory, not aspirational |
| Protocol-agnostic | No Raft, no IBFT, no consensus knowledge inside `faction` |
| Observer coverage | Every transition, query, and rejection reaches the `Observer` |
| Persistency-free | The machine never persists state; durability is the consumer's, via deterministic replay |
| Strict superset property | Phase N tests pass unchanged at Phase N+6 |

Each constraint is recorded as an Architecture Decision Record in
[docs/ADRs/](./ADRs/) — the authoritative statement of these properties.

---

## Phase 0 — Static membership (cluster bootstrapping)

**Status:** ✅ Complete  
**Published:** crates.io  

`faction` answers: *"Is the cluster ready to proceed?"*

A two-phase startup barrier that coordinates when a statically-known group of nodes
has reached participation and readiness quorum. Deterministic. Fully observable.
Queryable. Tested across every spawn/transport combination.

**Delivered:**

| Metric | Value |
|---|---|
| Core productive LOC | ~885 |
| Total tests | 277 |
| Code coverage | 100% |
| `(state, command)` pairs tested | All — explicitly |
| Crappy functions (CRAP score) | 0 |
| Unsafe code | 0 |
| System test combinations | 10 |
| Spawn models validated | 3 — Task, Thread, Process |
| Transport protocols validated | 4 — Memory, Channels, TCP, gRPC |

Full specification in [ARCHITECTURE.md](./ARCHITECTURE.md).

`PeerId` is currently `u64`. After Phase 6, it becomes a generic trait so
callers can use their own identifier type.

---

## Phase 1 — Dynamic membership: joining

**Status:** Planned  
**Target:** 4–6 weeks  
**Removes limitation:** L1 — static peer list  

`faction` answers: *"Can a new peer join the cluster?"*

The peer set is no longer fixed at construction. A peer can send a `Join` signal and
be admitted to the member set at runtime. Once admitted, its subsequent signals are
treated as valid member signals.

**New inputs:**
- `JoinRequested { peer_id }` — a peer requests admission
- `JoinApproved { peer_id }` — the caller approves the join request
- `JoinRejected { peer_id }` — the caller rejects the join request

**New outputs:**
- `EmitJoinRequest { peer_id }` — forward join request to caller for approval decision
- `MemberAdmitted { peer_id }` — peer admitted to member set
- `JoinDenied { peer_id }` — peer rejected

**Design invariant:** `faction` never decides admission policy. It signals the request
and acts on the caller's decision. Admission policy belongs to the protocol above.

**Gate:** 100% `(state, input)` coverage. Phase 2 does not begin until this gate is green.

---

## Phase 2 — Failure detection

**Status:** Planned  
**Target:** 4–6 weeks  
**Depends on:** Phase 1 complete  
**Removes limitation:** L2 — no liveness tracking  

`faction` answers: *"Is everyone still alive?"*

The membership set is now mutable and tracked. The machine monitors liveness using
SWIM-style probing: suspect → indirect probe → confirm or revive. Quorum is
re-evaluated as liveness changes.

**New states:**
- `Stable` — all members alive, quorum maintained
- `Degraded(SuspectSet)` — one or more members suspected
- `QuorumLost` — suspicion exceeds quorum threshold

**New inputs:**
- `ProbeAck { from }` — successful probe response
- `ProbeMiss { from }` — probe timeout
- `IndirectProbeAck { target, via }` — SWIM indirect confirmation
- `IndirectProbeMiss { target, via }` — SWIM indirect miss
- `Revive { node }` — suspected node re-confirmed

**New outputs (commands to caller):**
- `SendProbe { to }` — caller executes the probe
- `SendIndirectProbe { target, via }` — caller executes indirect probe
- `EmitQuorumLost` — notify consensus layer
- `EmitQuorumRestored` — notify consensus layer

The machine never probes. It emits probe commands. The caller executes them. Results
return as inputs. The Mealy invariant is maintained.

**Gate:** 100% `(state, input)` coverage. Phase 3 does not begin until this gate is green.

---

## Phase 3 — Single-node addition

**Status:** Planned  
**Target:** 4–6 weeks  
**Depends on:** Phase 2 complete  
**Removes limitation:** L3 — no addition protocol  

`faction` answers: *"Can we grow by one, safely?"*

Membership changes are now controlled via a commit/abort reconfiguration protocol.
The machine enforces a strict single-change-at-a-time invariant structurally — it is
impossible to request a second addition while one is in flight.

**Why this is distinct from Phase 1.** Phase 1 answers "is this peer allowed
to participate?" — it is a gatekeeper with no safety guarantees. Multiple
peers can join concurrently; the machine neither coordinates nor rolls back.
Phase 3 answers "does changing the member set preserve quorum safety?" — it
wraps the change in a commit/abort protocol with a mutual exclusion guard.
Where Phase 1 is permissive and stateless, Phase 3 is coordinated and atomic.

**New states:**
- `Reconfiguring(PendingAddition)` — one addition committed, further changes blocked

**New inputs:**
- `RequestAdd { node }` — caller requests addition
- `JoinAck { from }` — joining node confirmed membership snapshot
- `AddCommitted` — consensus layer committed the addition
- `AddAborted` — addition failed, rollback

**New outputs:**
- `SendMembershipSnapshot { to }` — send current membership to joining node
- `RejectAdd { reason }` — second change requested while one is in flight
- `EmitMemberAdded { node }` — committed, stable

**Core invariant:** at no point do two disjoint subsets of nodes each believe they form
a valid quorum. Single-at-a-time additions make this structurally impossible.

**Gate:** 100% `(state, input)` coverage. Phase 4 does not begin until this gate is green.

---

## Phase 4 — Single-node removal

**Status:** Planned  
**Target:** 4–6 weeks  
**Depends on:** Phase 3 complete  
**Removes limitation:** L4 — no removal protocol  

`faction` answers: *"Can we shrink by one, without breaking quorum?"*

Symmetric to Phase 3, but with harder invariants. The machine refuses removals that
would drop the live set below quorum threshold. This is not a runtime check — it is
a state transition that is structurally unavailable when the live set would be
insufficient.

**Why this is distinct from Phase 2.** Phase 2 answers "is this peer alive?"
— it observes and reports liveness. A dead peer stays in the member set;
the machine tracks suspicion but takes no action. Phase 4 answers "should this
peer be removed from the member set?" — it executes a coordinated state change
with quorum safety guarantees. Detection is passive observation. Removal is an
action with consequences.

**New inputs:**
- `RequestRemove { node }`
- `RemoveCommitted`
- `RemoveAborted`

**New outputs:**
- `RejectRemove { reason }` — explicit reason: `WouldBreakQuorum`, `NodeAlreadySuspected`,
  `ReconfigurationInFlight`
- `EmitMemberRemoved { node }`

**Degenerate cases — each is a first-class state transition, not an error path:**

| Input | State | Correct behaviour |
|---|---|---|
| Remove a suspected-dead node | `Degraded` | Valid — handled differently from removing a live node |
| Remove the last node in a minority partition | `Degraded` | Rejected — `WouldBreakQuorum` |
| Probe arrives after node is removed | Post-removal | Silently discarded via defined transition |

**Gate:** 100% `(state, input)` coverage. Phase 5 does not begin until this gate is green.

---

## Phase 5 — Membership epochs and rejoin

**Status:** Planned  
**Depends on:** Phase 4 complete  
**Removes limitation:** L5 — no epochs, no rejoin handling  

`faction` answers: *"Can the cluster survive membership changes across restarts
and prevent stale nodes from causing split-brain?"*

The machine gains a monotonic epoch counter, stale-epoch rejection, and explicit
rejoin handling for previously-removed nodes.

**Membership epochs.** Every committed change increments a monotonic epoch counter.
All membership messages carry the epoch. Stale-epoch inputs produce a defined
`StaleEpoch` transition — not a crash, not a silent ignore.

**Rejoin handling.** A previously-removed node can request to rejoin. The machine
distinguishes a new join from a rejoin — the state transitions differ explicitly.

**Split-brain prevention.** A node claiming quorum with a stale epoch is explicitly
rejected. The machine tracks the epoch at which each node last confirmed membership.

**New states:**
- `SplitSuspected` — minority partition suspected
- `Rejoining(PeerId)` — previously-removed node being re-admitted

**Gate:** 100% `(state, input)` coverage. Phase 6 does not begin until this gate is green.

---

## Phase 6 — Concurrent membership changes

**Status:** Planned  
**Depends on:** Phase 5 complete  
**Removes limitation:** L6 — single-change-at-a-time constraint  

`faction` answers: *"Can multiple membership changes be in flight simultaneously
under load?"*

The single-change-at-a-time constraint from Phases 3–4 is lifted. The machine
gains a bounded concurrent change queue with defined overflow behavior.

**Concurrent change sequencing.** Additions and removals are serialized into a
bounded queue. The caller provides the bound. Queue overflow produces a defined
`QueueFull` rejection, not a panic.

**New inputs:**
- `EnqueueAdd { node }` — caller requests addition; queued, not executed immediately
- `EnqueueRemove { node }` — caller requests removal; queued
- `Dequeued { node }` — change pulled from queue, ready for execution
- `FlushQueue` — caller drains the queue

**New outputs:**
- `QueueFull { reason }` — bounded queue exhausted
- `ChangeDequeued { node, kind }` — change ready for caller to process
- `QueueDrained` — all queued changes processed

**Gate:** 100% `(state, input)` coverage. Publication readiness review. crates.io
release candidate.

---

## Timeline

| Phase | Deliverable | Target | Status |
|---|---|---|---|
| 0 | Static membership, cluster bootstrapping | — | ✅ Complete |
| 1 | Dynamic joining | 4–6 weeks | Planned |
| 2 | Failure detection (SWIM) | 4–6 weeks | Planned |
| 3 | Single-node addition | 4–6 weeks | Planned |
| 4 | Single-node removal | 4–6 weeks | Planned |
| 5 | Membership epochs and rejoin | After Phase 4 | Planned |
| 6 | Concurrent membership changes | After Phase 5 | Planned |

---

## Publication readiness checklist

- [ ] Phases 1–6 complete with 100% `(state, input)` coverage
- [ ] System test matrix extended for all new states and transitions
- [ ] `no_std + alloc` verified at every phase boundary
- [ ] `#![deny(unsafe_code)]` enforced throughout
- [ ] CRAP score 0 across all crates
- [ ] Observer coverage — every new transition pair verified
- [ ] `NodeId` generic trait replacing `u64`
- [ ] README updated to reflect full dynamic membership capability
- [ ] ARCHITECTURE updated with Phase 1–6 specification
- [ ] At least one published sample integration (Raft or IBFT)
- [ ] crates.io metadata complete

---

## Hard rules

These rules are non-negotiable. They apply to every phase, every commit, every PR.

- No phase begins until the previous phase has 100% `(state, input)` coverage
- Each phase is a strict superset of the previous — Phase N tests pass unchanged at Phase 6
- No logic change without a failing test first
- No unsafe code — ever
- The machine never performs I/O — it only emits commands
- Every degenerate input in every state is a first-class transition, not an error path
- One struct per file
- CRAP score 0 before any phase is declared complete

---

## Paper — EuroSys / OSDI submission

**Status:** Planned  
**Target:** Parallel with Phase 4–6  
**Venue:** EuroSys 2027 (submission ~September 2026)  

Document the three contributions:
- The `faction` primitive and its design rationale
- The `(state × input)` complete coverage methodology
- Empirical validation across embedded and cloud targets

The paper is written in parallel with Phases 4 through 6 implementation.
The git history provides the quality trajectory data.
The system test matrix provides the empirical validation.
