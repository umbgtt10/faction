# Roadmap

**Crate:** `faction`  
**License:** MIT  
**Last updated:** 2026-11-13  
**Current status:** Phase 2 — failure detection landed (specification 0.3.0, unreleased; the crate 0.7.0 at the owner's word)

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

**Status:** ✅ Complete  
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

**Delivered:**

| Metric | Value |
|---|---|
| `(state, command)` coverage | 100% — explicitly |
| Unsafe code | 0 |
| Crappy functions (CRAP score) | 0 |
| System test cases | 134 |
| Join scenarios | 6 + natural timeout, across Task/Thread × {memory, channels, TCP, gRPC} and Process × {TCP, gRPC} |
| Spawn models validated | 3 — Task, Thread, Process |
| Transport protocols validated | 4 — Memory, Channels, TCP, gRPC |

`Members` are carried in each `State` (genesis ∪ admitted); admission stays a local
control-plane decision, with policy owned by the caller. The scenario Phase 1 deferred —
`join-raises-quorum` — runs in Phase 3, as a committed addition raising the required count.

---

## Cross-cutting — Specification & conformance suite

**Status:** specification 0.3.0 written, in [`spec/`](../spec/SPEC.md).
**Not a numbered phase:** this spans every phase rather than removing one
limitation; it does not fit the L1-L7 framing above and does not shift any
phase's numbering.

`faction` is specified as a standard component, the crate one implementation of
it, so a non-Rust port has a machine-checkable conformance target instead of
only the Rust source to read (`P0-ADR-SpecificationIsNormativeCrateIsReference`).
The specification's four parts — schema, decision table, prose and vectors — are
written by hand and versioned together; its 0.1.0 is Phase 1's vocabulary,
joining included; its 0.2.0 adds Phase 3's addition, and its 0.3.0 Phase 2's failure detection. Every phase changes the specification first, then
the crate (`P2-ADR-SpecifiedBeforeAnyNewPhase`). The specification's independent
model, `crates/spec-model/`, checks each phase's vectors before the crate runs them
(`P1-ADR-IndependentModelOfTheSpecification`).

---

## Phase 2 — Failure detection

**Status:** ✅ Complete — specification 0.3.0; the crate 0.7.0, unreleased
**Depends on:** Phases 1 and 3 complete — built after Phase 3
**Removes limitation:** L2 — no liveness tracking

`faction` answers: *"Is everyone still alive?"*

A concluded node runs SWIM's failure detector, the caller keeping its clock and its
wire (`P1-ADR-LivenessReportedNotStaged`). It names, one period at a time, the member to
probe, round-robin; a member that misses its probe is probed through others; one that
misses as many periods running as the node's suspicion threshold is suspected; one
heard from again is revived. Quorum is the members not suspected against the count in
effect, and the node says when it is lost and restored. Liveness is something the view
reports, never a stage; each node judges for itself; a suspect stays a member — only a
removal (Phase 4) takes one away.

**New configuration:** the suspicion threshold — how many periods running a member must
miss before it is suspected, each node's own.

**New commands:**
- `ProbeDue` — a probe period began
- `PeerHeard { peer_id }` — a sign of life from the member, direct or relayed
- `ProbeMissed { peer_id }` — the member missed its probe
- `IndirectProbesMissed { peer_id }` — the member missed the period, through every helper asked

**New outcomes:**
- `SendProbe { peer_id }`, `NothingToProbe`, `SendIndirectProbes { peer_id, via }`
- `MissCounted { peer_id, misses }`, `PeerSuspected { peer_id }`, `PeerRevived { peer_id }`
- `AliveNoted`, `AlreadySuspected`, `SelfIgnored`
- `QuorumLost`, `QuorumRestored`

The machine never probes: it names whom to probe and whom to ask, and the caller's
answers return as commands. The Mealy invariant is maintained.

**Delivered:**

| Metric | Value |
|---|---|
| Specification | 0.3.0 — 46 rows, 78 vectors, every row exercised |
| Property invariants | Every other member probed within one round; a suspicion only at the threshold; the quorum held exactly while the members not suspected reach the count |
| In-process scenarios | 10 — time passing at a test's word |
| System scenarios | 5, in all 10 spawn and transport cases, members silenced and links cut by the harness |

**Gate:** 100% `(state, input)` coverage. Phase 4 does not begin until this gate is green.

---

## Phase 3 — Single-node addition

**Status:** ✅ Complete — specification 0.2.0, released in 0.6.0
**Depends on:** Phase 1 complete — built ahead of Phase 2
**Removes limitation:** L3 — no addition protocol

`faction` answers: *"Can a running cluster grow by one, safely?"*

A concluded node grows by one through the layer that commits changes above it
(`P1-ADR-AdditionThroughTheLayerThatCommits`). One addition is in flight at a time,
refused structurally while it is: the stage admits no second request.

**Why this is distinct from Phase 1.** Phase 1 answers "is this peer allowed
to participate?" — it is a gatekeeper with no safety guarantees. Multiple
peers can join concurrently; the machine neither coordinates nor rolls back.
Phase 3 answers "does changing the member set preserve quorum safety?" — it
wraps the change in a commit/abort protocol with a mutual exclusion guard.
Where Phase 1 is permissive and stateless, Phase 3 is coordinated and atomic.

**New stage:**
- `Reconfiguring` — concluded, one addition in flight; its peer state is `Bootstrapped`,
  its view names the addition

**New commands:**
- `AdditionRequested { peer_id }` — the caller asks the node to coordinate an addition
- `MembershipAcknowledged { peer_id }` — the newcomer holds the membership it was sent
- `AdditionCommitted { peer_id }` — the layer above committed the addition, on every node
- `AdditionAborted { peer_id }` — the layer above abandoned it
- `RequiredCountChanged { required_count }` — the count in effect, reported with the commit

**New outcomes:**
- `SendMembership { peer_id }` — send the newcomer the members and the required count
- `ProposeAddition { peer_id }` — propose the addition to the layer above
- `AdditionWithdrawn`, `AcknowledgementIgnored`, `AbortIgnored`, `RequiredCountSet`

**Core invariant:** at no point do two disjoint subsets of nodes each believe they form
a valid quorum. One addition at a time, counted on every node only once committed,
makes this structurally impossible.

**Delivered:**

| Metric | Value |
|---|---|
| Specification | 0.2.0 — 29 rows, 62 vectors, every row exercised |
| `(stage, command)` coverage | All 65 pairs — explicitly |
| In-process scenarios | 6 — a lost membership, a lost acknowledgement, a commit reaching nodes still bootstrapping, a commit for another peer, a count lowered while collecting, an abort |
| System scenarios | 5, in all 10 spawn and transport cases |

**Gate:** 100% `(state, input)` coverage. Phase 4 does not begin until this gate is green.

---

## Phase 4 — Single-node removal

**Status:** Planned  
**Target:** 4–6 weeks  
**Depends on:** Phases 2 and 3 complete  
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

*To settle in Phase 4's proposal:* `NodeAlreadySuspected` as a reason to refuse clashes
with this phase's own table, which makes the removal of a suspect valid; and Phase 2 has
left the suspected members and the quorum in the view for the removal rules to read.

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
| 1 | Dynamic joining | 4–6 weeks | ✅ Complete (core logic); HW validation in progress |
| cross-cutting | Specification & conformance suite | Before Phase 3 | Specification 0.3.0 written |
| 2 | Failure detection (SWIM) | 4–6 weeks | ✅ Complete (specification 0.3.0; the crate 0.7.0, unreleased) |
| 3 | Single-node addition | 4–6 weeks | ✅ Complete (specification 0.2.0, released in 0.6.0) |
| 4 | Single-node removal | 4–6 weeks | Planned |
| 5 | Membership epochs and rejoin | After Phase 4 | Planned |
| 6 | Concurrent membership changes | After Phase 5 | Planned |

---

## Phase 1 publication readiness checklist (interim, near-term)

Publishing the Phase-1-inclusive release to crates.io does **not** wait for
Phases 2-6 — see the checklist below this one for that. This interim gate
is what the cross-cutting specification work above is itself waiting on:

- [ ] `etheram-ibft-embassy` stages 4-6 green on real NUCLEO hardware
- [ ] `etheram-raft-embassy` stages 4-6 green on real NUCLEO hardware
- [ ] `etheram-raft`'s own Phase 1 integration designed and landed
- [ ] `(state, command)` coverage remains 100% after all three integrations
- [ ] crates.io metadata complete for the Phase-1-inclusive release

**Progress, 2026-07-28:** `etheram-raft` (on its own
`feat/faction-0.4-integration` branch, not yet merged to its `main`) has
re-pinned from Faction's git+branch reference to the released `0.4.0` and
re-confirmed its own stages 1, 2, and 5 green — design is done and the
integration builds clean against the real release. Not checking the box
yet: "landed" reads as merged to `etheram-raft`'s own `main`, which is
still gated on `etheram-raft-embassy`'s own re-pin and both embassy repos'
hardware stages 4-6.

## Full publication readiness checklist (Phases 1–6, longer-term)

This is the eventual, complete-dynamic-membership publication milestone —
distinct from, and later than, the interim checklist above.

- [ ] Phases 1–6 complete with 100% `(state, input)` coverage
- [ ] System test matrix extended for all new states and transitions
- [ ] `no_std + alloc` verified at every phase boundary
- [ ] `#![forbid(unsafe_code)]` enforced throughout
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
