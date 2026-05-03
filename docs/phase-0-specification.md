# Phase 0 — Static membership (cluster bootstrapping)

**Status:** Complete  
**Lines of code (productive):** 1,165  
**Tests:** 145 core + 23 validation + 33 protocol + 9 protocol-validation + 54 system = 264 total  
**Crappy functions:** 0  
**Code coverage (productive):** 100%

---

## Architecture

`faction` implements a **deterministic, two-phase cluster bootstrapping state machine** — a
startup barrier that coordinates when a group of nodes is ready to proceed.

The machine is a pure Mealy model: `output = F(state, input)`. It performs no I/O, has no
side effects, and is trivially replayable. Every transition is observable through the
`Observer` trait.

Decision logic is split into three focused step structs — `PingingStep`, `ReadyStep`,
and `LocalCompletionStep` — each handling exactly one kind of observation. State structs
are pure data containers with minimal trait implementations.

---

## State machine

### States

| State | Meaning | Carries |
|---|---|---|
| `Initial` | Freshly created, no action taken yet | Nothing — unit struct |
| `Pinging` | Collecting participation signals from peers | `pinging_peers: Vec<PeerId>`, `collecting_peers: Vec<PeerId>` |
| `Collecting` | Local participation complete, collecting readiness signals | `collecting_peers: Vec<PeerId>`, `pinged_peers: Vec<PeerId>` |
| `Bootstrapped` | Quorum reached, cluster is ready (terminal) | `pinged_peers: Vec<PeerId>`, `collected_peers: Vec<PeerId>` |
| `TimedOut` | Deadline expired before quorum (terminal) | `pinging_peers: Vec<PeerId>`, `collecting_peers: Vec<PeerId>` |

### Commands

| Command | Effect |
|---|---|
| `ParticipationObserved { peer_id }` | A peer sent a participation signal |
| `ReadyObserved { peer_id }` | A peer sent a readiness signal |
| `LocalParticipationCompleted` | The local node signals its participation is done |
| `DeadlineExpired` | External deadline timer fired |
| `Probe` | Query current cluster view without mutation |

### Outcomes

| Outcome | Meaning |
|---|---|
| `ParticipationAccepted { peer_id }` | Participation signal from a member |
| `ReadyAccepted { peer_id }` | Readiness signal from a member |
| `DuplicateParticipationIgnored { peer_id }` | Duplicate participation signal |
| `DuplicateReadyIgnored { peer_id }` | Duplicate readiness signal |
| `NonMemberIgnored { peer_id }` | Signal from a peer not in the member set |
| `LocalParticipationCompleted` | Local node finished its own participation |
| `BroadcastLocalReady` | Local readiness should be broadcast to peers |
| `Concluded { mode }` | Machine concluded (`Bootstrapped` or `TimedOut`) |

---

## Behaviors

### Deduplication

Every observation checks whether the peer is already in the confirmed set for its
phase. Duplicate signals produce `Duplicate*Ignored` outcomes and do not mutate state.

### Quorum

Quorum is a configurable threshold passed at construction via `QuorumPolicy::new(count)`.
The machine does not know what quorum means — it only checks whether the confirmed peer
count meets or exceeds the threshold.

Quorum is checked when a new (non-duplicate) peer is confirmed and the resulting count
meets or exceeds the threshold. Duplicate signals never re-trigger quorum. Quorum is
checked in the collecting phase only (via `ReadyStep`) and during local completion
(via `LocalCompletionStep`).

### Signal filtering

Every command goes through two gates, in order:

1. **Non-member gate** — signals from peers not in the static member set are rejected
   with `NonMemberIgnored`. Membership is checked against `Config`'s peer list, which
   is fixed at construction.

2. **Duplicate gate** — signals from already-confirmed peers produce `Duplicate*Ignored`
   outcomes. New peers have their identity added to the confirmed set.

### Step structs

| Struct | Handles | Quorum? |
|---|---|---|
| `PingingStep` | `ParticipationObserved` in pinging phase | Never |
| `ReadyStep` | `ReadyObserved` in collecting phase (and accumulates in pinging) | Yes, with threshold |
| `LocalCompletionStep` | `LocalParticipationCompleted` | Yes, always |

### Cluster view

Current state is queryable at any time via `Probe`, with zero side effects. The response
includes: current state (`PeerState`), exit mode (`Conclusion`), pinging peers, collecting
peers, whether pinging is complete, and the required quorum count.

### Observer

Every transition is observable through the `Observer` trait, which has three methods:

| Method | Triggered by |
|---|---|
| `observe(command, transition)` | Accepted command |
| `observe_query(command, cluster_view)` | Probe |
| `observe_rejection(command, cluster_view, admissible)` | Rejected command |

The observer receives the full before/after transition, not just individual outcomes.

### Process results

`process(command)` returns one of three results:

| Result | Meaning |
|---|---|
| `Accepted { outcomes, cluster_view }` | Command executed, new state and outcomes returned |
| `Rejected { cluster_view, admissible }` | Command rejected, state unchanged, admissible commands listed |
| `Probed { cluster_view, admissible }` | Probe executed, state unchanged |

---

## Validation harness

`faction-core-validation` provides two tools:

### ScenarioHarness

Multi-node deterministic simulation with configurable peer sets. Each node is an
independent `Faction` instance. The harness exposes:
- `apply_participation(node_index, peer_id)` — feed a participation signal
- `apply_ready(node_index, peer_id)` — feed a readiness signal
- `complete_local_participation(node_index)` — trigger local participation completion
- `expire_deadline(node_index)` — trigger deadline
- `cluster_view(node_index)` — query a node's current cluster view

### ClusterSimulation

Event-driven simulation with broadcast queue. Simulates realistic multi-node startup
with automatic broadcast propagation.

### Property-based tests

Property-based invariants verified by `proptest` across thousands of random command
sequences:

| Invariant |
|---|
| Exit happens at most once |
| Confirmed peer counts never decrease |
| Duplicate and non-member inputs never mutate state |
| Deadline and quorum both lead to correct concluded states |
| Exit mode never changes after exit |
| Required count never changes |
| Model reference implementation matches the real machine for all random sequences |

---

## Limitations (removed one by one in Phases 1–5)

| # | Limitation | Removed in |
|---|---|---|
| L1 | **Static membership.** The peer list is fixed at construction. No peer can join, leave, or be added after initialization. | Phase 1 |
| L2 | **No liveness tracking.** Once the machine concludes, it is terminal. No ping, no probe, no suspicion, no revival. | Phase 2 |
| L3 | **Single-node addition requires protocol.** Adding a node mid-flight is not supported. No join handshake, no reconfiguration state. | Phase 3 |
| L4 | **Single-node removal requires protocol.** Removing a node mid-flight is not supported. No leave signal, no quorum-preserving removal check. | Phase 4 |
| L5 | **No epochs, no concurrent changes.** Membership has no version counter. Concurrent additions/removals are not sequenced. No split-brain prevention. | Phase 5 |
| L6 | **No durable state.** The machine is in-memory only. No crash recovery, no persisted membership log. | Future |
| L7 | **No generic identity.** Peer IDs are `u64`. No `NodeId` trait, no address resolution. | Future |
