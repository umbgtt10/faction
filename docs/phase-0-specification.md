# Phase 0 — Static membership (cluster readiness)

**Status:** Complete  
**Lines of code (productive):** 1,312  
**Tests:** 192 core + 34 validation = 216 total  
**Crappy functions:** 0 / 114  
**Code coverage:** 99.7% (100% effective, one `const fn` branch is a coverage tool artifact)

---

## Architecture

`faction` implements a **deterministic, two-phase cluster readiness state machine** — a startup
barrier that coordinates when a group of nodes is ready to proceed.

The machine is a pure Mealy model: `output = F(state, input)`. It performs no I/O, has no
side effects, and is trivially replayable. Every transition is observable through the `Observer`
trait.

Core decision logic lives in a single struct: **`ObservedStep`**. It encapsulates freshness
classification, deduplication, quorum detection, and outcome computation. The state
structs (`Pinging`, `Collecting`, `Bootstrapped`, `TimedOut`, `Initial`) are pure data
containers with minimal trait implementations.

---

## State machine

### States

| State | Meaning | Carries |
|---|---|---|
| `Initial` | Freshly created, no action taken yet | Nothing — unit struct |
| `Pinging` | Collecting participation signals from peers | `pinged_peers: Vec<PeerId>`, `collected_peers: Vec<PeerId>` |
| `Collecting` | Local participation complete, collecting readiness signals | `collected_peers: Vec<PeerId>`, `pinged_peers_count: usize` |
| `Bootstrapped` | Quorum reached, cluster is ready (terminal) | `pinged_peers_count: usize`, `collected_peers_count: usize` |
| `TimedOut` | Deadline expired before quorum (terminal) | `pinged_peers_count: usize`, `collected_peers_count: usize` |

### Inputs (commands)

| Command | Effect |
|---|---|
| `ParticipationObserved { peer_id, freshness, current_marker }` | A peer sent a participation signal |
| `ReadyObserved { peer_id, freshness, current_marker }` | A peer sent a readiness signal |
| `LocalParticipationCompleted` | The local node signals its own participation is done |
| `DeadlineExpired` | External deadline timer fired |
| `Probe` | Query current cluster view without mutation |

### Outputs (outcomes)

| Outcome | Meaning |
|---|---|
| `ParticipationAccepted { peer_id }` | Participation signal from a member, timely |
| `DelayedParticipationAccepted { peer_id }` | Participation signal, delayed within margin |
| `StaleParticipationIgnored { peer_id }` | Participation signal too old |
| `DuplicateParticipationIgnored { peer_id }` | Duplicate signal from already-confirmed peer |
| `ReadyAccepted { peer_id }` | Readiness signal from a member, timely |
| `DelayedReadyAccepted { peer_id }` | Readiness signal, delayed within margin |
| `StaleReadyIgnored { peer_id }` | Readiness signal too old |
| `DuplicateReadyIgnored { peer_id }` | Duplicate readiness signal |
| `NonMemberIgnored { peer_id }` | Signal from a peer not in the member set |
| `LocalParticipationCompleted` | Local node finished its own participation |
| `BroadcastLocalReady` | Local readiness should be broadcast to peers |
| `ReadyQuorumReached` | Threshold of readiness signals met |
| `Exited { mode }` | Machine exited (`Bootstrapped` or `TimedOut`) |

---

## Behaviors

### Freshness classification

Each observation carries a freshness marker. The configurable `FreshnessPolicy` classifies it:

| Classification | Condition |
|---|---|
| `Timely` | `observed_marker == current_marker` |
| `DelayedWithinMargin` | `0 < current_marker - observed_marker <= max_delay` |
| `Stale` | `current_marker - observed_marker > max_delay` or `observed_marker > current_marker` (future-dated) |

The policy is immutable and configured at construction via `FreshnessPolicy::new(max_delay)`.

### Quorum

Quorum is a configurable threshold passed at construction via `QuorumPolicy::new(required_count)`.
The machine does not know what quorum means — it only checks `confirmed_peers.len() >= required_count`.

Quorum is checked exactly once: when a new peer is confirmed and the count meets or exceeds the
threshold. Duplicate or stale signals never re-trigger quorum detection.

### Signal filtering

Every command goes through three gates, in order:

1. **Non-member gate** — signals from peers not in the static member set are rejected with
   `NonMemberIgnored`. Membership is checked against the `Config`'s peer list, which is fixed
   at construction and never changes.

2. **Freshness gate** — stale signals produce `Stale*Ignored` outcomes. Non-stale signals proceed.

3. **Duplicate gate** — signals from already-confirmed peers produce `Duplicate*Ignored` outcomes.
   New peers have their identity added to the confirmed set.

### Cluster view

Current state is queryable at any time via `Probe`, with zero side effects. The response
includes: current state (`PeerState`), exit mode, list of confirmed peers for each phase,
whether local participation is complete, and the required quorum count.

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

`faction-validation` provides two tools:

### ScenarioHarness

Multi-node deterministic simulation with configurable peer sets. Each node is an independent
`Faction` instance. The harness exposes:
- `apply_participation(node_index, peer_id, freshness)` — feed a participation signal to a node
- `apply_ready(node_index, peer_id, freshness)` — feed a readiness signal
- `complete_local_participation(node_index)` — trigger local participation completion
- `expire_deadline(node_index)` — trigger deadline
- `cluster_view(node_index)` — query a node's current cluster view
- `advance_to(marker)` — advance the freshness marker for all nodes

### ClusterSimulation

Event-driven simulation with broadcast queue and marker advancement. Simulates realistic
multi-node startup with automatic broadcast propagation, delayed signals, and stale signals.

### Property-based tests

Property-based invariants verified by `proptest` across thousands of random command sequences:

| Invariant |
|---|
| Exit happens at most once |
| Confirmed peer counts never decrease |
| Stale, duplicate, and non-member inputs never mutate state |
| Deadline and quorum both lead to correct exited states with matching exit modes |
| Exit mode never changes after exit |
| Required count never changes |
| Model reference implementation matches the real machine for all random sequences |

---

## Limitations (removed one by one in Phases 1–5)

| # | Limitation | Removed in |
|---|---|---|
| L1 | **Static membership.** The peer list is fixed at construction. No peer can join, leave, or be added after initialization. `Config::is_member()` is a constant-time table lookup over an immutable list. | Phase 1 |
| L2 | **No liveness tracking.** Once the machine exits (bootstrapped or timed out), it is terminal. No ping, no probe, no suspicion, no revival. The cluster either formed or didn't. | Phase 2 |
| L3 | **Single-node addition requires protocol.** Adding a node mid-flight is not supported. There is no join handshake, no membership snapshot exchange, no reconfiguration state. | Phase 3 |
| L4 | **Single-node removal requires protocol.** Removing a node mid-flight is not supported. There is no leave signal, no quorum-preserving removal check, no graceful departure. | Phase 4 |
| L5 | **No epochs, no concurrent changes.** Membership has no version counter. Concurrent additions and removals are not sequenced. There is no split-brain prevention, no rejoin handling. | Phase 5 |
| L6 | **No durable state.** The machine is in-memory only. No crash recovery, no persisted membership log, no replay from storage. | Future |
| L7 | **No generic identity.** Peer IDs are `u64`. The machine has no `NodeId` trait, no address resolution, no pluggable identity model. | Future |
