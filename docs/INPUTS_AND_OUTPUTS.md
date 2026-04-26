# Inputs and Outputs

The `ClusterReadiness` state machine accepts a fixed set of inputs and produces a fixed set of outputs on each transition.

---

## Inputs

| Input | Variant | Payload | Description |
|---|---|---|---|
| `ParticipationObserved` | Signal | `peer_id`, `freshness`, `current_marker` | A participation observation from a remote peer. The machine classifies it by freshness and, if accepted, increments the Phase 1 count. |
| `ReadyObserved` | Signal | `peer_id`, `freshness`, `current_marker` | A readiness observation from a remote peer. The machine classifies it by freshness and, if accepted, increments the Phase 2 count. |
| `LocalParticipationCompleted` | Event | — | The local node signals that its own participation is done. Triggers the transition from Phase 1 to Phase 2 and increments the Phase 2 count for the local peer. |
| `DeadlineExpired` | Event | — | A timer fires indicating the readiness deadline has passed. Causes the machine to exit with `Deadline` regardless of quorum state. |

---

## Outputs

All outputs are defined in `ClusterReadinessOutput`.

### Accepted Signals

| Output | Meaning | Triggered By |
|---|---|---|
| `ParticipationAccepted { peer_id }` | A timely participation signal was accepted and the Phase 1 count was incremented. | `ParticipationObserved` (timely) |
| `ReadyAccepted { peer_id }` | A timely readiness signal was accepted and the Phase 2 count was incremented. | `ReadyObserved` (timely) |

### Delayed Signals (Still Accepted)

| Output | Meaning | Triggered By |
|---|---|---|
| `DelayedParticipationAccepted { peer_id }` | A delayed (but within margin) participation signal was accepted. | `ParticipationObserved` (delayed) |
| `DelayedReadyAccepted { peer_id }` | A delayed (but within margin) readiness signal was accepted. | `ReadyObserved` (delayed) |

### Ignored Signals

| Output | Meaning | Triggered By |
|---|---|---|
| `DuplicateParticipationIgnored { peer_id }` | A participation signal from a peer that was already counted was ignored. | `ParticipationObserved` (duplicate) |
| `DuplicateReadyIgnored { peer_id }` | A readiness signal from a peer that was already counted was ignored. | `ReadyObserved` (duplicate) |
| `StaleParticipationIgnored { peer_id }` | A participation signal outside the acceptable freshness window was ignored. | `ParticipationObserved` (stale) |
| `StaleReadyIgnored { peer_id }` | A readiness signal outside the acceptable freshness window was ignored. | `ReadyObserved` (stale) |
| `NonMemberIgnored { peer_id }` | A signal from a peer that is not in the configured peer set was ignored. | `ParticipationObserved` or `ReadyObserved` |

### Lifecycle Events

| Output | Meaning | Triggered By |
|---|---|---|
| `LocalParticipationCompleted` | The local node's participation has been recorded. | `LocalParticipationCompleted` |
| `BroadcastLocalReady` | The machine requests that the local readiness be broadcast to peers. | `LocalParticipationCompleted` (when Phase 2 begins) |
| `ReadyQuorumReached` | The Phase 2 quorum threshold has been satisfied. | Any input that increments Phase 2 count past the threshold |
| `ReadinessExited { mode }` | The machine has exited readiness coordination. | Quorum reached or deadline expired |

### Exit Modes

| Mode | Meaning |
|---|---|
| `Quorum` | The machine exited because a readiness quorum was reached. |
| `Deadline` | The machine exited because the deadline expired before quorum was reached. |

---

## Transition Model

```
                    ParticipationObserved
                    (accepted / delayed)
                         │
                         ▼
  ┌────────────────────────────────┐
  │        Phase 1 Active          │
  │  (collecting participation)    │
  └────────────────────────────────┘
                         │
              LocalParticipationCompleted
                         │
                         ▼
  ┌────────────────────────────────┐
  │        Phase 2 Active          │
  │  (collecting readiness)        │
  └────────────────────────────────┘
                         │
              ReadyQuorumReached
              or DeadlineExpired
                         │
                         ▼
  ┌────────────────────────────────┐
  │     Exited (Quorum/Deadline)   │
  │       (no more transitions)    │
  └────────────────────────────────┘