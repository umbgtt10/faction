# Architecture

**Status:** Phase 1 — dynamic joining landed  
**Core productive LOC:** ~895  
**Total tests:** 506  
**Code coverage:** 100%  
**Crappy functions:** 0  
**Unsafe code:** 0  

---

## Overview

`faction` implements a **deterministic, two-phase cluster bootstrapping state machine**.
It is a startup barrier: it coordinates when a group of nodes with a known genesis
membership is ready to proceed as a cluster. Since Phase 1 the genesis set is a seed,
not a ceiling — a newcomer can join a running cluster through a membership axis
orthogonal to the bootstrapping progression (see the Phase 1 records in [ADRs/](./ADRs/)).

The machine is a pure Mealy model:

```
output = F(state, input)
```

It performs no I/O. It has no side effects. It holds no timers. Every transition is a
pure function of the current state and the incoming command. Any execution is fully
replayable from its input log.

---

## Why a Mealy machine?

The Mealy model was chosen deliberately over alternatives for four reasons:

**1. Complete testability.** Because `output = F(state, input)`, the entire behavior
of the machine is captured by the set of `(state, command)` pairs. This set is finite
and enumerable. The test suite can — and does — cover every pair explicitly. Coverage
is not a percentage. It is a proof.

**2. Deterministic replay.** Any production incident is reproducible by replaying the
input log against the initial state. No timing dependencies, no thread scheduling
artifacts, no network non-determinism. The machine sees the same inputs and produces
the same outputs every time.

**3. Clean separation of concerns.** The machine computes transitions. The caller
executes effects. Network I/O, timers, persistence, and process management are the
caller's responsibility. `faction` has no opinion on any of them.

**4. Formal verifiability.** The Mealy model maps directly onto a formal
specification: each transition is a single action over `(state, input)`, so the
machine's behaviour can be stated and checked in a specification language (e.g. TLA+)
using the same vocabulary as the implementation. (No TLA+ model ships in the repo
today — the point is that the design is amenable to one.)

---

## Why two phases?

Cluster bootstrapping involves two distinct signals:

* **Participation** — "I am alive and joining the cluster."
* **Readiness** — "I have finished my startup work. I am ready to proceed."

A single-phase design conflates them. Every node races to confirm readiness, and
the first node to collect enough signals can declare quorum and exit — before it has
ever signalled its own participation, let alone finished its own startup.

This is not subtle. In any real cluster some nodes start faster than others. The
fast node hears readiness pings from slower peers, tallies quorum, and announces
success — while its own initialization is still in progress. The cluster has
"converged" with a member that isn't ready.

The two-phase design eliminates this structurally:

```text
SINGLE-PHASE (broken):

  Peer 2: ──Ready──►
  Peer 3: ──Ready──►
  Peer 4: ──Ready──►
  Peer 1:  [still starting up...]  ◄── declares quorum anyway

  All it takes is one fast node that hears readiness before finishing its own work.

TWO-PHASE (fixed):

  Participation ("I'm alive")           Readiness ("I'm ready")
  ┌──────────────────────────┐          ┌──────────────────────────┐
  │ Gate: LocalParticipation │  ───►    │ Quorum checked here      │
  │ Completed required       │          │                          │
  │ No quorum check          │          │ Node can't enter Phase 2 │
  │                          │          │ without passing the gate │
  └──────────────────────────┘          └──────────────────────────┘
```

**Phase 1 — Pinging.** The node collects *participation* signals ("I'm alive")
from peers. It cannot leave this phase until it declares its own participation
complete via `LocalParticipationCompleted`. No quorum check happens here.

**Phase 2 — Collecting.** The node collects *readiness* signals ("I'm ready").
Quorum is only checked in this phase. A node that hasn't passed through Phase 1
cannot reach Phase 2, and therefore cannot exit with `Bootstrapped` — regardless
of how many readiness signals it hears.

The invariant: **a node that hasn't finished its own startup can never declare
quorum.** Not by policy. Not by convention. By construction.

---

## State machine specification

### Construction

```rust
let config = Config::new(peer_id, peers, QuorumPolicy::new(required), suspicion_threshold);
let machine = Faction::new(config, observer);
```

`Config` holds the static peer list, the quorum threshold and the suspicion threshold —
how many probe periods running a member must miss before it is suspected. `QuorumPolicy`
wraps an integer — the machine only checks "does the confirmed count meet or exceed this?"
`Faction::new` takes both plus an `Observer` trait object. Construction produces a
machine in the `Initial` state, its view `Fresh`: the configuration's peers as its
members, its required count, and nothing confirmed.

### States

| State | Meaning | Carries |
|---|---|---|
| `Initial` | Freshly created, no signal taken | Its members |
| `Pinging` | Collecting participation signals from peers | Members, the participation and readiness accepted so far, and whether the deadline was missed |
| `Collecting` | Local participation complete, collecting readiness | Members, the participation accepted, the readiness accepted — the node's own among it — and whether the deadline was missed |
| `Bootstrapped` | Quorum reached — cluster is ready (the conclusion) | Members, the peer sets at the conclusion, and its members' liveness |
| `Reconfiguring` | Concluded, coordinating one change, an addition or a removal — its peer state `Bootstrapped` | The concluded node and the change in flight |
| `Removed` | Removed from its own cluster — admits a probe alone | Its own id: its view is what it held when removed, but itself and its liveness |

Every stage carries the node's members — the configuration's peers, then each peer
admitted or added, less each removed — and the required count in effect, starting at
the configuration's. Membership decisions never change the stage, but for a node's own
removal, which leaves it `Removed`; `JoinApproved` is admitted in every stage but
`Reconfiguring`, where no admission goes around a change in flight, and `Removed`.

`Bootstrapped` is the only conclusion; `Reconfiguring` is a concluded node with a
change in flight, an addition or a removal (`P1-ADR-AdditionThroughTheLayerThatCommits`,
`P1-ADR-RemovalThroughTheLayerThatCommits`); `Removed` is a node whose own removal
committed, and no longer a member of its own cluster. A missed deadline is **not** a state: it is
recorded as a non-terminal fact — surfaced as `PeerState::TimedOut` through a derived
view flag — while the node stays in `Pinging`/`Collecting`, keeps its retries, and can
still converge if readiness arrives late.

Early readiness signals accumulate in `Pinging` before `LocalParticipationCompleted`
arrives — the machine does not discard signals that arrive before their phase. This
means a fast peer that signals readiness before the local node finishes participating
is not penalized; its signal waits and is counted in `Collecting`.

The terminal state is not a silent sink. A `Bootstrapped` node stops driving its own
progress, but it still accepts `ParticipationObserved` and re-advertises its readiness
(`AcknowledgeRejoin`) — so a peer that missed the original broadcast and is still
pinging can always recover. A removed node answers a probe alone: no member waits on
it, since every member applies the same commit.

Liveness is not a state either (`P1-ADR-LivenessReportedNotStaged`). A concluded node
probes its members and suspects those it cannot reach, but a suspicion changes what its
view shows, never what it admits.

### Commands

| Command | Meaning |
|---|---|
| `ParticipationObserved { peer_id }` | A peer sent a participation signal |
| `ReadyObserved { peer_id }` | A peer sent a readiness signal |
| `LocalParticipationCompleted` | The local node finished its own participation |
| `DeadlineExpired` | External deadline timer fired |
| `JoinRequested { peer_id }` | A newcomer asked to join |
| `JoinApproved { peer_id }` | The caller admitted the newcomer |
| `JoinRejected { peer_id }` | The caller refused the newcomer |
| `AdditionRequested { peer_id }` | The caller asks a concluded node to coordinate an addition |
| `MembershipAcknowledged { peer_id }` | The newcomer holds the membership it was sent |
| `AdditionCommitted { peer_id }` | The layer above committed the addition — reported to every node |
| `AdditionAborted { peer_id }` | The layer above abandoned the addition |
| `RemovalRequested { peer_id }` | The caller asks a concluded node to coordinate a member's removal |
| `RemovalCommitted { peer_id }` | The layer above committed the removal — reported to every node |
| `RemovalAborted { peer_id }` | The layer above abandoned the removal |
| `RequiredCountChanged { required_count }` | The required count in effect — reported with the change it belongs to |
| `ProbeDue` | A probe period began |
| `PeerHeard { peer_id }` | A sign of life from the member — an answer to a probe, direct or relayed |
| `ProbeMissed { peer_id }` | The member missed its probe |
| `IndirectProbesMissed { peer_id }` | The member missed the period, directly and through every member asked |
| `Probe` | Query current state without mutation |

### Outcomes

| Outcome | Meaning |
|---|---|
| `ParticipationAccepted { peer_id }` | New participation signal from a member |
| `ReadyAccepted { peer_id }` | New readiness signal from a member |
| `DuplicateParticipationIgnored { peer_id }` | Already-confirmed peer sent again |
| `DuplicateReadyIgnored { peer_id }` | Already-confirmed peer sent again |
| `NonMemberIgnored { peer_id }` | Signal from a peer not in the member set |
| `LocalParticipationCompleted` | Local node completed its participation |
| `BroadcastLocalReady` | Local readiness should be broadcast to peers |
| `AcknowledgeRejoin { peer_id }` | A concluded node should re-advertise readiness to a still-pinging member |
| `DeadlineMissed { confirmed_count }` | The deadline fired before quorum — recorded, non-terminal |
| `EmitJoinRequest { peer_id }` | Forward the newcomer's request to whoever decides |
| `MemberAdmitted { peer_id }` | The newcomer is now a member |
| `DuplicateMemberIgnored { peer_id }` | The peer approved was already a member |
| `JoinDenied { peer_id }` | The newcomer was refused; nothing changed |
| `Concluded { mode }` | The node concluded bootstrapping (`mode` is `Bootstrapped`) |
| `SendMembership { peer_id }` | Send the newcomer the members and the required count |
| `ProposeAddition { peer_id }` | Propose the addition to the layer that commits |
| `AdditionWithdrawn { peer_id }` | The addition in flight was aborted |
| `AcknowledgementIgnored { peer_id }` | An acknowledgement not awaited, or a repeat |
| `AbortIgnored { peer_id }` | An abort not naming the change in flight |
| `ProposeRemoval { peer_id }` | Propose the removal to the layer that commits |
| `RemovalRefused { peer_id }` | The quorum cannot spare the member: the members not suspected would fall below the count |
| `RemovalWithdrawn { peer_id }` | The removal in flight was aborted |
| `MemberRemoved { peer_id }` | The peer is no longer a member — the node itself among them, which is then `Removed` |
| `RequiredCountSet { required_count }` | The required count in effect is now this |
| `SendProbe { peer_id }` | Probe the member — the next in turn |
| `NothingToProbe` | The node is its only member |
| `SendIndirectProbes { peer_id, via }` | Ask members of those listed to probe it |
| `MissCounted { peer_id, misses }` | A missed period counted, below the suspicion threshold |
| `PeerSuspected { peer_id }` | The member has missed as many periods running as the threshold |
| `PeerRevived { peer_id }` | A suspect heard from again |
| `AliveNoted { peer_id }` | A member alive heard from; its missed periods forgotten |
| `AlreadySuspected { peer_id }` | A suspect missed again; nothing changed |
| `SelfIgnored { peer_id }` | A command of liveness naming the node itself |
| `QuorumLost` / `QuorumRestored` | The command took the members not suspected across the required count |

The call that concludes says so: its outcomes end with `Concluded { mode: Bootstrapped }`.
The view says so from then on: its peer state is `Bootstrapped`.
`Bootstrapped` is the only conclusion.

### Data flow

```text
Command
  │
  ▼
the stage's step ── answers nothing? ─► ProcessResult::Probed for a Probe,
  │                                     ProcessResult::Rejected otherwise (no mutation)
  ▼
Gate 1: non-member? ──► NonMemberIgnored
  │
  ▼
Gate 2: duplicate? ───► Duplicate*Ignored
  │
  ▼
Step struct ──────────► outcomes + new_state
  │
  ▼
ProcessResult::Accepted { cluster_view, admissible, outcomes }
```

Whichever result a command gets, it is reported — `Observer::observe(&command, &result)` —
and then returned.

### Process results

`process(command)` returns one of three structured results. There is no error type.
Every input produces a defined, handled result.

| Result | Meaning | State mutated? |
|---|---|---|
| `Accepted { cluster_view, admissible, outcomes }` | Command executed | Yes |
| `Rejected { cluster_view, admissible }` | Command not valid in current state | No |
| `Probed { cluster_view, admissible }` | Probe executed | No |

All three results carry the list of admissible commands — the caller always knows what
is valid next, on every path. This eliminates defensive programming on the caller side.

---

## Behavioral specification

### Signal filtering pipeline

Every command passes through two gates before mutation occurs:

**Gate 1 — Non-member check.** Signals from peers not in the static member set produce
`NonMemberIgnored`. Membership is checked against the node's members — the
configuration's peers, and every peer admitted since — in every stage that admits
the signal.

**Gate 2 — Duplicate check.** Signals from already-confirmed peers produce
`Duplicate*Ignored`. The duplicate check is per-phase: a peer confirmed in the pinging
phase is not considered a duplicate in the collecting phase. A peer who participated
and then signalled readiness produces two distinct `Accepted` outcomes.

### Quorum

Quorum is a configurable threshold, set at construction via `QuorumPolicy::new(count)`
and changed afterwards by `RequiredCountChanged` alone. The machine does not interpret
quorum — it checks whether the confirmed count meets or exceeds the count in effect.

Quorum is checked in exactly three places:

1. `ReadyStep` — after a new (non-duplicate) readiness signal is confirmed
2. `LocalCompletionStep` — when the local node signals completion
3. `Collecting`, after a new required count — a count the collecting peers already
   reach concludes

Duplicate signals never re-trigger quorum checks. Quorum is never checked before the
node's own completion: readiness accumulates while `Pinging` and counts at completion.
This ensures the two-phase invariant holds under all inputs.

Once concluded, the node holds its quorum while its members not suspected number the
count in effect or more. A suspicion, a raised count or the removal of a member not
suspected may lose it; a revival, a member admitted or added, or a lowered count may
restore it, and no removal ever does; the command that crosses says so, last among its
outcomes.

### Liveness

A concluded node probes its members one period at a time, as SWIM's failure detector
does, without its gossip (`P1-ADR-LivenessReportedNotStaged`). The caller keeps the clock:

1. `ProbeDue` names the member after the one probed last, round-robin, the node skipped —
   every other member once every N−1 periods.
2. `ProbeMissed` lists the members not suspected that may probe the missed one, from the
   member after it; the caller asks the first k.
3. `IndirectProbesMissed` counts a missed period; at the configuration's suspicion
   threshold the member is suspected.
4. `PeerHeard` revives a suspect, and forgets an alive member's missed periods.

A suspect stays a member: only a removal, committed by the layer above, takes a member
away, and its liveness goes with it. How tolerant a node is, is its deployment's — the
period and timeouts, the helpers asked, and the threshold it is created with.

### Removal

A concluded cluster shrinks by one through the layer that commits changes above it
(`P1-ADR-RemovalThroughTheLayerThatCommits`), coordinated as an addition is but with no
handshake:

1. `RemovalRequested` of a member not suspected the quorum cannot spare — the members
   not suspected would number fewer than the required count without it — is refused,
   `RemovalRefused`; of any other member, the node itself among them, it is proposed at
   once, `ProposeRemoval`, and the node is `Reconfiguring`. A suspect's removal is never
   refused for the quorum.
2. `RemovalCommitted`, reported to every node in every stage, takes the peer out of every
   list the node holds — its members, the peers it gathered, its suspects and missed
   periods; the member probed last, if it was the one, is forgotten. It is never refused,
   and may lose the quorum on a node that still heard the member.
3. `RemovalAborted` withdraws the removal in flight.
4. A node whose own removal commits is `Removed`.

Who is removed, and when, is the caller's: `faction` never removes a member for its
silence. The required count is the caller's too, reported with the commit — lowered
first, if the removal needs it.

### Step structs

Decision logic is split into focused step structs. Each handles one kind of
observation.

| Struct | Handles | Checks quorum? |
|---|---|---|
| `PingingStep` | `ParticipationObserved` before the node's own completion | Never |
| `ReadyBeforeCompletionStep` | `ReadyObserved` before the node's own completion | Never |
| `ReadyStep` | `ReadyObserved` while `Collecting` | Yes, with threshold |
| `LocalCompletionStep` | `LocalParticipationCompleted` | Yes, always |
| `MembershipStep` | `JoinRequested`, `JoinApproved`, `JoinRejected`, `AdditionCommitted`, `RemovalCommitted` of another peer, `RequiredCountChanged`, in every stage that admits them | `Collecting` checks the new count |

What a concluded node answers whatever its change in flight — a peer still
bootstrapping, a membership decision — is `Bootstrapped::answer`; `Reconfiguring`
reuses it, refusing an admission and a second request of either kind, and carries the
change's acknowledgement, abort and commit itself. A node's own removal, answered alike
in every stage that admits it, each of them takes through `Removed::own_removal`.

Readiness before the node's own completion is accepted by `ReadyBeforeCompletionStep`,
with no quorum check.

This design means that adding a new command type cannot accidentally affect existing
command handling. Each step struct is independently testable.

### Cluster view

The cluster view is queryable at any time via `Probe`, with zero side effects. It
returns:

- Current peer state (`PeerState`) — `TimedOut` is reported here as a derived
  "missed deadline" flag, distinct from being concluded
- Pinging peers — peers observed in Phase 1
- Collecting peers — peers observed in Phase 2
- Whether pinging phase is complete
- Whether the deadline has been missed (`deadline_missed`)
- The required count in effect
- Members — the configuration's peers, then every admitted or added peer, in arrival order, less each removed
- The reconfiguration — the change in flight: none, an addition requested of a peer or acknowledged by it, or a peer's removal requested
- The suspected members, in the order suspected, and each member's missed periods
- The member probed last
- The quorum — not tracked before the conclusion nor once removed, held or lost between

The queryability model means external monitors can poll any node's view without
perturbing the machine. Composition of multiple nodes' cluster views gives an
aggregated picture of the cluster from multiple vantage points.

---

## Observability

Every call is reported to the `Observer` trait, once: `observe(&command, &result)`,
with the command and the `ProcessResult` it was answered — an accepted command, a
rejected one and a probe alike, by the time the result is returned. The result carries
the whole view and the admissible commands, so a report is not a log line but the
answer itself, and a node's reports, in order, are its log (`spec/SPEC.md` §3). The
view a command found is the one the report before it carried.

The observer is told and answers nothing, so it cannot change a result.

The observer is a trait. It can be wired to structured logging, metrics pipelines,
distributed tracing, test assertions, or audit logs. The machine does not know or care
which. Multiple observers can be composed.

This design choice — observer as a trait, not a logger — means observability is a
first-class architectural concern, not an afterthought. It also means the test suite
can assert on every transition without modifying production code.

---

## Validation harness

### Unit test coverage

The crates/core/ test suite covers every (state, command) pair explicitly. The complete matrix is documented and tested in
[transition_matrix_tests.rs](../validation/ladder/tests/matrix/state_transition_matrix_tests.rs) — every combination named, every outcome asserted.

### Property-based invariants

`proptest` verifies the following invariants across thousands of random command
sequences:

| Invariant |
|---|
| Exit happens at most once |
| Confirmed peer counts never decrease |
| Duplicate and non-member inputs never mutate state |
| A missed deadline is recorded but never concludes the machine |
| Exit mode never changes after exit |
| A state's admissible set equals exactly its accepted commands plus `Probe` |
| The specification's independent model agrees with the machine for all sequences |

The last invariant is the strongest: the specification's independent model
(`crates/spec-model/`), written from the prose and the decision table and depending on nothing
of the crate, answers every random input sequence beside the machine, and every result
must be identical in the specification's encoding. Any divergence is a specification
bug or an implementation bug — both are caught immediately.

### Multi-node scenarios

`faction-core-validation` provides two multi-node simulation tools:

**`ScenarioHarness`** — deterministic multi-node simulation. Each node is an
independent `Faction` instance. The harness exposes typed methods to feed signals to
specific nodes and inspect their state. Used for scenario-based tests where the exact
sequence of signals matters.

**`ClusterSimulation`** — event-driven simulation with broadcast queue. Simulates
realistic multi-node startup with automatic propagation of broadcast outputs. Used for
testing emergent cluster behaviour under realistic message delivery.

### System test matrix

The system test suite validates the complete protocol runtime — including real OS
processes, real TCP connections, and real gRPC channels — across all valid combinations
of spawn model, transport, and timer:

| Spawn | Transport | Timer | Status |
|---|---|---|---|
| Task | In-memory | In-memory, Real | ✅ |
| Task | Channels | In-memory, Real | ✅ |
| Task | TCP | Real | ✅ |
| Task | gRPC | Real | ✅ |
| Thread | In-memory | In-memory, Real | ✅ |
| Thread | Channels | In-memory, Real | ✅ |
| Thread | TCP | In-memory, Real | ✅ |
| Thread | gRPC | Real | ✅ |
| Process | TCP | Real | ✅ |
| Process | gRPC | Real | ✅ |

Process-based tests spawn real OS processes. Each process binds a real port. The test
harness waits for listener readiness before allowing connections. Timer delays are
configurable per spawn model to account for OS scheduling overhead.

---

## Specification & conformance

`faction` is specified in [`spec/`](../spec/SPEC.md), normatively: a JSON Schema
data model, a JSON decision table, prose for what JSON cannot say, and
conformance vectors, versioned together and written by hand. This crate is one
implementation of it; where the two disagree, the specification is right
(`P0-ADR-SpecificationIsNormativeCrateIsReference`). A second, `crates/spec-model/`, is
independent of the crate: it reads the decision table and answers each command
by the first row that applies, checks every vector before the crate runs it, and
is the judge the crate is checked against over random command sequences
(`P1-ADR-IndependentModelOfTheSpecification`).

The data model keeps the vocabulary of `Command`, `Outcome` and
`ProcessResult`, and is mapped to them by the crate's conformance tests, never
derived from them. The decision table says what each stage admits and gives
every `(stage, command)` its row; each vector names the rows it exercises, and
every row is exercised. Outcomes are an ordered sequence and peers are listed in
arrival order, so a vector is checked by deep equality. No invariant rests on the
type system: admission is decided when a command arrives, so every vector runs
on every conforming implementation.

This connects to, but is distinct from, the "Formal verifiability" property
above: a TLA+ model would check the Mealy machine's logic exhaustively; the
specification and its vectors check that independent implementations agree
with this one.

---

## Design decisions

**No generic `NodeId` (current limitation).** Peer IDs are currently `u64`. A generic
`NodeId` trait is planned for a future phase. This is a known limitation (L7) and does
not affect correctness.

**`alloc` but not `std`.** The machine requires heap allocation for peer sets. Pure
`no_std` without `alloc` would require fixed-size arrays and a compile-time node count,
which is impractical for dynamic cluster sizes. The `no_std + alloc` combination is the
correct middle ground: works on embedded targets with an allocator, works on WASM,
works on cloud.

**No `async`.** The machine is synchronous. It is the caller's responsibility to drive
the machine from an async context if needed. This keeps the machine dependency-free and
usable from any execution model — tokio, async-std, Embassy, bare metal, or threaded.

**A concluded node is terminal but not a silent sink.** `Bootstrapped` is the only
terminal state, and it stops driving its own progress — but it still answers a
still-pinging peer by re-advertising its readiness (`AcknowledgeRejoin`). This closes
the class of bug where a node that reached quorum went quiet and stranded a peer that
missed the original broadcast. A missed deadline is likewise non-terminal: it is
recorded (`DeadlineMissed`) and the node stays receptive, so a cluster can recover
from a premature deadline instead of dead-ending in a `TimedOut` state.

**Persistency-free.** The machine holds state but never persists it — it writes nothing
to disk and reads no ambient input, so its state is reconstructed by deterministic
replay of the input log. Durability, and any snapshot that bounds replay, is the
caller's responsibility.

Each decision on this page is recorded as an Architecture Decision Record — one
property per file — under [`docs/ADRs/`](./ADRs/), the authoritative rationale.

---

## Limitations

The following limitations are present in Phase 0. Each is removed in a specific
subsequent phase.

| # | Limitation | Removed in |
|---|---|---|
| L1 | **Static membership.** The peer list is fixed at construction. No peer can join or be added after initialization. | Phase 1 |
| L2 | **No liveness tracking.** Once concluded, the machine is terminal. No failure detection, no suspicion, no revival. | Phase 2 |
| L3 | **No single-node addition protocol.** Joining mid-flight requires a commit/abort reconfiguration protocol not yet implemented. | Phase 3 |
| L4 | **No single-node removal protocol.** Leaving or being removed requires a quorum-preserving removal check not yet implemented. | Phase 4 |
| L5 | **No epochs or rejoin handling.** Membership has no version counter. Stale nodes are not rejected. Previously-removed nodes cannot rejoin. No split-brain prevention. | Phase 5 |
| L6 | **Single-change-at-a-time.** Concurrent additions and removals are not sequenced. No bounded queue, no defined overflow behavior. | Phase 6 |
| L7 | **No generic identity.** Peer IDs are `u64`. No `NodeId` trait, no address resolution abstraction. | Future |
