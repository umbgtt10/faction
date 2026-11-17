# Roadmap

**Crate:** `faction`  
**License:** MIT  
**Last updated:** 2026-11-17  
**Current status:** Phase 5 — epochs and rejoin landed (specification 0.5.0, for the crate's 0.9.0); the chaos
validation built (specification 0.6.0, for the crate's 0.10.0); a provisional node coordinating
no change (specification 0.7.0, for the crate's 0.11.0); every answer a numbered receipt and every node an
instance (specification 0.8.0, for the crate's 0.12.0)

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

**Status:** specification 0.4.0 written, in [`spec/`](../spec/SPEC.md).
**Not a numbered phase:** this spans every phase rather than removing one
limitation; it does not fit the L1-L7 framing above and does not shift any
phase's numbering.

`faction` is specified as a standard component, the crate one implementation of
it, so a non-Rust port has a machine-checkable conformance target instead of
only the Rust source to read (`P0-ADR-SpecificationIsNormativeCrateIsReference`).
The specification's four parts — schema, decision table, prose and vectors — are
written by hand and versioned together; its 0.1.0 is Phase 1's vocabulary,
joining included; its 0.2.0 adds Phase 3's addition, its 0.3.0 Phase 2's failure detection, and its 0.4.0 Phase 4's removal. Every phase changes the specification first, then
the crate (`P2-ADR-SpecifiedBeforeAnyNewPhase`). The specification's independent
model, `crates/spec-model/`, checks each phase's vectors before the crate runs them
(`P1-ADR-IndependentModelOfTheSpecification`).

---

## Phase 2 — Failure detection

**Status:** ✅ Complete — specification 0.3.0, released in 0.7.0
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

**Status:** ✅ Complete — specification 0.4.0, released in 0.8.0
**Depends on:** Phases 2 and 3 complete
**Removes limitation:** L4 — no removal protocol

`faction` answers: *"Can we shrink by one, without breaking quorum?"*

A concluded cluster shrinks by one through the layer that commits changes above it,
coordinated as an addition is but with no handshake — the member most worth removing is
the one that cannot answer (`P1-ADR-RemovalThroughTheLayerThatCommits`). The node asked
for a removal refuses it when the quorum cannot spare the member: a member not
suspected, without whom the members not suspected would number fewer than the required
count. A suspect's removal it never refuses for the quorum. One change, an addition or a
removal, is in flight at a time — the view's addition became its reconfiguration. Every
node applies the commit, in every stage, the peer leaving every list it holds; a node
whose own removal commits is `Removed`, a sixth stage admitting a probe alone.

**Why this is distinct from Phase 2.** Phase 2 answers "is this peer alive?" — it
observes and reports liveness, and a suspect stays a member. Phase 4 answers "should
this peer leave the member set?" — a coordinated change with a quorum check. Detection
is observation; removal is an action, and who is removed, and when, stays the caller's.

**New commands:**
- `RemovalRequested { peer_id }` — coordinate the removal of a member
- `RemovalCommitted { peer_id }` — the layer above committed it, reported to every node
- `RemovalAborted { peer_id }` — the layer above abandoned it

**New outcomes:**
- `ProposeRemoval { peer_id }`, `RemovalRefused { peer_id }`
- `RemovalWithdrawn { peer_id }`, `MemberRemoved { peer_id }`

The drafts' three reasons to refuse were settled in the proposal: `WouldBreakQuorum` is
`RemovalRefused`; a change in flight is the stage, answered `Rejected`; and
`NodeAlreadySuspected` was dropped, since removing a suspect is what removal is mostly
for.

**Degenerate cases — each is a first-class state transition, not an error path:**

| Input | State | Behaviour |
|---|---|---|
| Remove a suspect | concluded, suspects in the view | Proposed — never refused for the quorum |
| Remove a member the quorum cannot spare | concluded | `RemovalRefused` |
| A removed member's answer arrives | any concluded node | `NonMemberIgnored`, as a non-member's |
| The commit reaches a node that still hears the member | concluded | `MemberRemoved`, then `QuorumLost` |
| The node's own removal commits | any stage but `Removed` | `MemberRemoved`, and the node is `Removed` |

**Delivered:**

| Metric | Value |
|---|---|
| Specification | 0.4.1 — 56 rows, 93 vectors, every row exercised, each vector kept as released |
| Property invariant | A removal proposed exactly when the quorum can spare the member |
| In-process scenarios | 7 |
| System scenarios | 5, in all 10 spawn and transport cases |

**Gate:** 100% `(state, input)` coverage. Phase 5 does not begin until this gate is green.

---

## Phase 5 — Membership epochs and rejoin

**Status:** ✅ Complete — specification 0.5.0, for the crate's 0.9.0
**Depends on:** Phase 4 complete
**Removes limitation:** L5 — no epochs, no rejoin handling

`faction` answers: *"Can the cluster survive membership changes across restarts,
and keep a stale node from counting?"*

Every membership a node holds carries an epoch: the number the layer above gave the
change that made it, which the commit brings (`P1-ADR-TheEpochComesWithTheCommit`). The
node takes it and never counts, since commits reach nodes at different moments, and a
commit at or below its epoch is one it holds already. A member's participation,
readiness or an addition's acknowledgement from behind is refused, and answered with the
membership; liveness carries no epoch, so a slow member is never suspected for being
slow. A membership above the node's epoch is adopted in every stage — a change in flight
given up, a removed node it names taken back to `Initial`, a node it does not name
removed. It is the first growth under the specification's own rule: epoch 0 is omitted
wherever it would stand, and every vector released at 0.4.0 passes unchanged.

**Settled in the proposal, against the drafts above:**
- The epoch is the commit's, not a counter `faction` keeps.
- `StaleEpoch` is `StaleEpochIgnored`, for claims alone; liveness is never refused.
- No `SplitSuspected` or `Rejoining` stage: a node behind learns it only when handed the
  membership, and adopts it in the same call; rejoining is `Initial` and bootstrapping
  again. The epoch each node last confirmed is each signal's own.
- Whether a membership is genuine is the caller's, the Notary bookkeeper's once built.

**New command:** `MembershipReceived { members, required_count, epoch }`; the epoch on
`ParticipationObserved`, `ReadyObserved`, `MembershipAcknowledged`, `AdditionCommitted`,
`RemovalCommitted` and `RequiredCountChanged`, and on the configuration and the view.

**New outcomes:** `StaleEpochIgnored`, `CommitIgnored`, `MembershipIgnored`,
`MembershipAdopted`, `ChangeAbandoned`.

**In the reference protocol,** the epoch rides `Ping`, `Ready`, `Membership` and
`MembershipAcknowledged`; a running node takes a membership as a newcomer does, and a
removed node a coordinator invites back is made a newcomer again — the owner's choice
over an invitation that would guess the layer above's next number.

**Delivered:**

| Metric | Value |
|---|---|
| Specification | 0.5.0 — 70 rows, 108 vectors, every row exercised, the 93 of 0.4.0 unchanged |
| Property invariants | A node's epoch never decreases; a claim from behind never counts |
| In-process scenarios | 5 |
| System scenarios | 5, in all 10 spawn and transport cases |

**Gate:** 100% `(state, input)` coverage. Phase 6 does not begin until this gate is green.

---

## Cross-cutting — Chaos validation

**Status:** ✅ Complete (specification 0.6.0, for the crate's 0.10.0); the plan as built is
`docs/history/[IMPLEMENTED] CHAOS-VALIDATION.md`  
**Order:** after Phase 5, before Phase 6  
**Not a numbered phase:** it removes no limitation; it checks the phases built so far
together, and it is how we learn what Phase 6 must guarantee once changes interleave.

`faction` answers: *"Whatever the network and the timers do, and in whatever order the
changes land, does every node end where the committed changes say it must — and is it
safe along the way?"*

A seed draws a scenario: a cluster, membership changes from several coordinators in partly
random order, and the faults of the network and of the timers. The harness plays the layer
above, committing one change at a time in epoch order, late or twice, never reordered.
Every invariant is watched while the faults play; once they heal, every node is checked
against the end state the committed log implies — computed, never observed. A deterministic
engine plays the scenarios on task nodes and a simulated clock, a seed replaying exactly; a
real one on thread and process nodes over the four transports, on the machine's clock. A
failure is replayed with every node's log, shrunk to the smallest scenario that still fails,
and recorded as a regression stage 1 replays.

**What it settled.** Phase 2 wired its fault layer for an assessment it left open: whether
some convergence-critical signal, once lost, is never sent again — a permanent stall below
100 % loss. The chaos validation is that assessment, over every fault it was to run — loss,
duplication, delay, reordering, partitions, one-way links, a kind of message dropped — and
more, from the bootstrap on: a lost signal nothing repeats leaves a node short of the end
state once the faults heal, which the oracle reports. No lost signal stalled a cluster.
The protocol's findings were of another kind — in the table below — and every stall the
soaks met was the harness's own. Three limits stand: under faults that never heal, liveness
is the caller's, the deadline and the freshness margin tuned against the loss rate; a lying
peer, corruption among its ways, is the Notary's; and a new transport, UDP or WebSocket, is
breadth, not a failure mode the fault layer does not inject already.

**Delivered:**

| Metric | Value |
|---|---|
| Specification | 0.5.1 — the layer above owes a node its commits in epoch order; 0.6.0 — a newcomer holds its membership provisionally |
| Findings, fixed | A membership taken by a node it was not addressed to; a newcomer answering from behind with a membership not yet committed; a member's ping answered with a broadcast to every member |
| Deterministic engine | 300 seeds and 8 regressions in stage 1; fresh seeds on demand (`just chaos-soak`) |
| Real engine | 3 seeds in each of the 6 thread and process cases, on demand (`just chaos`) |

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
| cross-cutting | Specification & conformance suite | Before Phase 3 | Specification 0.8.0 written |
| 2 | Failure detection (SWIM) | 4–6 weeks | ✅ Complete (specification 0.3.0, released in 0.7.0) |
| 3 | Single-node addition | 4–6 weeks | ✅ Complete (specification 0.2.0, released in 0.6.0) |
| 4 | Single-node removal | 4–6 weeks | ✅ Complete (specification 0.4.0, released in 0.8.0) |
| 5 | Membership epochs and rejoin | After Phase 4 | ✅ Complete (specification 0.5.0, for the crate's 0.9.0) |
| cross-cutting | Chaos validation | After Phase 5, before Phase 6 | ✅ Complete (specification 0.6.0, for the crate's 0.10.0) |
| cross-cutting | Numbered receipts | Before Phase 6 | ✅ Complete (specification 0.8.0, for the crate's 0.12.0) |
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
