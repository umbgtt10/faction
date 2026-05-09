# Architecture

**Status:** Phase 0 — Complete  
**Productive LOC:** 1,165  
**Total tests:** 275  
**Code coverage:** 100%  
**Crappy functions:** 0  
**Unsafe code:** 0  

---

## Overview

`faction` implements a **deterministic, two-phase cluster bootstrapping state machine**.
It is a startup barrier: it coordinates when a group of nodes with known, static
membership is ready to proceed as a cluster.

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

**4. Formal verifiability.** The Mealy model maps directly to formal specification.
Each TLA+ action corresponds to one machine transition. The specification and the
implementation share the same vocabulary.

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
let config = Config::new(peer_id, peers, QuorumPolicy::new(required));
let machine = Faction::new(config, observer);
```

`Config` holds the static peer list and quorum threshold. `QuorumPolicy` wraps an
integer — the machine only checks "does the confirmed count meet or exceed this?"
`Faction::new` takes both plus an `Observer` trait object. Construction produces a
machine in the `Initial` state with an empty `ClusterView`.

### States

| State | Meaning | Carries |
|---|---|---|
| `Initial` | Freshly created, no action taken | Nothing — unit struct |
| `Pinging` | Collecting participation signals from peers | In-flight participation and readiness sets |
| `Collecting` | Local participation complete, collecting readiness | In-flight readiness and completed participation sets |
| `Bootstrapped` | Quorum reached — cluster is ready (terminal) | Final peer sets at time of exit |
| `TimedOut` | Deadline expired before quorum (terminal) | Peer sets at time of expiry |

Early readiness signals accumulate in `Pinging` before `LocalParticipationCompleted`
arrives — the machine does not discard signals that arrive before their phase. This
means a fast peer that signals readiness before the local node finishes participating
is not penalized; its signal waits and is counted in `Collecting`.

Terminal states (`Bootstrapped`, `TimedOut`) return `false` from `accept()`. This makes
`step()` structurally unreachable on a concluded machine. The compiler enforces this —
it is not a runtime check.

### Commands

| Command | Meaning |
|---|---|
| `ParticipationObserved { peer_id }` | A peer sent a participation signal |
| `ReadyObserved { peer_id }` | A peer sent a readiness signal |
| `LocalParticipationCompleted` | The local node finished its own participation |
| `DeadlineExpired` | External deadline timer fired |
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

Conclusion is computed from the `ClusterView`, not emitted as a standalone outcome.
Callers inspect `cluster_view.conclusion()` to determine whether the machine reached
`Bootstrapped` or `TimedOut`.

### Data flow

```text
Command
  │
  ├─ Probe? ──────────► ProcessResult::Probed (no mutation)
  │
  ├─ accept() false? ─► ProcessResult::Rejected { admissible }
  │
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
Observer.observe()
  │
  ▼
ProcessResult::Accepted { outcomes, cluster_view }
```

### Process results

`process(command)` returns one of three structured results. There is no error type.
Every input produces a defined, handled result.

| Result | Meaning | State mutated? |
|---|---|---|
| `Accepted { outcomes, cluster_view }` | Command executed | Yes |
| `Rejected { cluster_view, admissible }` | Command not valid in current state | No |
| `Probed { cluster_view, admissible }` | Probe executed | No |

`Rejected` includes the list of admissible commands — the caller always knows what
is valid at any point. This eliminates defensive programming on the caller side.

---

## Behavioral specification

### Signal filtering pipeline

Every command passes through two gates before mutation occurs:

**Gate 1 — Non-member check.** Signals from peers not in the static member set produce
`NonMemberIgnored`. Membership is checked against the `Config` peer list, which is
fixed at construction. This gate is unconditional — it fires regardless of state.

**Gate 2 — Duplicate check.** Signals from already-confirmed peers produce
`Duplicate*Ignored`. The duplicate check is per-phase: a peer confirmed in the pinging
phase is not considered a duplicate in the collecting phase. A peer who participated
and then signalled readiness produces two distinct `Accepted` outcomes.

### Quorum

Quorum is a configurable threshold set at construction via `QuorumPolicy::new(count)`.
The machine does not interpret quorum — it checks whether the confirmed count meets or
exceeds the threshold.

Quorum is checked in exactly two places:

1. `ReadyStep` — after a new (non-duplicate) readiness signal is confirmed
2. `LocalCompletionStep` — when the local node signals completion

Duplicate signals never re-trigger quorum checks. Quorum is never checked in the
pinging phase. This ensures the two-phase invariant holds under all inputs.

### Step structs

Decision logic is split into three focused step structs. Each handles exactly one
kind of observation. There is no branching on a command kind enum within a step struct.

| Struct | Handles | Checks quorum? |
|---|---|---|
| `PingingStep` | `ParticipationObserved` in pinging phase | Never |
| `ReadyStep` | `ReadyObserved` in any phase | Yes, with threshold |
| `LocalCompletionStep` | `LocalParticipationCompleted` | Yes, always |

This design means that adding a new command type cannot accidentally affect existing
command handling. Each step struct is independently testable.

### Cluster view

The cluster view is queryable at any time via `Probe`, with zero side effects. It
returns:

- Current peer state (`PeerState`)
- Exit mode if concluded (`Conclusion`)
- Pinging peers — peers observed in Phase 1
- Collecting peers — peers observed in Phase 2
- Whether pinging phase is complete
- Required quorum count

The queryability model means external monitors can poll any node's view without
perturbing the machine. Composition of multiple nodes' cluster views gives an
aggregated picture of the cluster from multiple vantage points.

---

## Observability

Every transition is delivered to the `Observer` trait. The observer receives the full
before-and-after context — not just a log line.

| Method | Triggered by |
|---|---|
| `observe(command, transition)` | Any accepted command |
| `observe_query(command, cluster_view)` | Any `Probe` command |
| `observe_rejection(command, cluster_view, admissible)` | Any rejected command |

A `Transition` carries the full before-and-after context: the previous `ClusterView`,
the `Vec<Outcome>` produced by the step, and the new `ClusterView`. No partial
snapshots. No guesswork.

The observer is a trait. It can be wired to structured logging, metrics pipelines,
distributed tracing, test assertions, or audit logs. The machine does not know or care
which. Multiple observers can be composed.

This design choice — observer as a trait, not a logger — means observability is a
first-class architectural concern, not an afterthought. It also means the test suite
can assert on every transition without modifying production code.

---

## Validation harness

### Unit test coverage

The core/ test suite covers every (state, command) pair explicitly. The complete matrix is documented and tested in
[transition_matrix_tests.rs](./core/tests/transition_matrix/state_transition_matrix_tests.rs) — every combination named, every outcome asserted.

### Property-based invariants

`proptest` verifies the following invariants across thousands of random command
sequences:

| Invariant |
|---|
| Exit happens at most once |
| Confirmed peer counts never decrease |
| Duplicate and non-member inputs never mutate state |
| Deadline and quorum both lead to correct concluded states |
| Exit mode never changes after exit |
| Required quorum count never changes |
| Model reference implementation agrees with the real machine for all sequences |

The last invariant is the strongest: a simplified reference implementation of the
machine is maintained alongside the production implementation. For every random input
sequence, both must produce identical outputs. Any divergence is a specification bug
or an implementation bug — both are caught immediately.

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

**Terminal states are structurally unreachable after conclusion.** Rather than checking
`is_concluded()` at every call site, the design uses `accept()` to make `step()`
unreachable on concluded machines. This is enforced by the type system, not by
convention.

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
