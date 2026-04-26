# faction

**faction** is a `no_std` + `alloc` Rust workspace for protocol-independent cluster readiness coordination — a two-phase startup barrier that tracks participation and readiness quorums across a known peer set, with support for delayed signals and deadline fallback.

It is designed for **embedded**, **distributed**, and **deterministic testing** environments where nodes must coordinate readiness before proceeding to consensus or application logic.

---

## Workspace

The project is split into two crates:

| Crate | Package | Description |
|---|---|---|
| `core/` | `faction` | The core state machine — cluster readiness coordination with freshness classification, quorum tracking, and observer hooks. |
| `validation/` | `faction-validation` | Deterministic scenario harness and test infrastructure for simulating multi-node readiness sequences. |

---

## `faction` — Core State Machine

### Overview

`ClusterReadiness` is a state machine that coordinates startup readiness across a known set of peers. It progresses through two phases:

1. **Phase 1 — Participation**: Nodes observe each other's participation signals. Once a quorum of participation observations is reached, the node enters Phase 2.
2. **Phase 2 — Readiness**: Nodes broadcast readiness via `LocalParticipationCompleted` and collect remote readiness observations. Once readiness quorum is reached, the machine exits with `ReadinessExitMode::Quorum`.

A **deadline** mechanism provides fallback: if `DeadlineExpired` is applied before quorum is reached, the machine exits with `ReadinessExitMode::Deadline`.

### Key Types

| Type | Description |
|---|---|
| `ClusterReadiness` | The state machine — call `apply(input)` to drive transitions. |
| `ClusterReadinessConfig` | Configuration: local peer ID, peer set, quorum policy, freshness policy. |
| `ClusterReadinessInput` | One of `ParticipationObserved`, `ReadyObserved`, `LocalParticipationCompleted`, `DeadlineExpired`. |
| `ClusterReadinessOutput` | Observability events emitted by each transition (accepted, ignored, stale, quorum reached, etc.). |
| `ClusterReadinessSnapshot` | Immutable snapshot of the machine's current state (lifecycle phase, counts, exit mode). |
| `ClusterReadinessTransition` | A full state transition record: previous snapshot → outputs → new snapshot. |
| `FreshnessPolicy` | Classifies observations as `Timely`, `DelayedWithinMargin`, or `Stale` based on a configurable `max_delay`. |
| `QuorumPolicy` | Determines whether a count satisfies the quorum threshold. |
| `OutputBatch` | A container for outputs produced by a single `apply()` call. |

### Freshness Classification

Observations carry a `freshness` (logical timestamp) compared against the observer's `current_marker`:

| Age | Classification |
|---|---|
| `age == 0` | `Timely` |
| `0 < age <= max_delay` | `DelayedWithinMargin` — still accepted |
| `age > max_delay` or future marker | `Stale` — ignored |

### Observer Hook

`ClusterReadinessObserver` receives every `(input, transition)` pair. Use it for logging, metrics, or trace capture. A `NoOpClusterReadinessObserver` is provided for production use when observability is not needed.

### Example

```rust
use faction::cluster_readiness::ClusterReadiness;
use faction::cluster_readiness_config::ClusterReadinessConfig;
use faction::cluster_readiness_input::ClusterReadinessInput;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_cluster_readiness_observer::NoOpClusterReadinessObserver;
use faction::quorum_policy::QuorumPolicy;
use alloc::boxed::Box;

let config = ClusterReadinessConfig::new(
    0,                          // local peer ID
    vec![0, 1, 2, 3, 4],       // peer set
    QuorumPolicy::new(4),       // quorum threshold (4 out of 5)
    FreshnessPolicy::new(2),    // max delay margin
);

let mut machine = ClusterReadiness::new(
    config,
    Box::new(NoOpClusterReadinessObserver),
);

// Observe a participation signal from peer 1
let batch = machine.apply(ClusterReadinessInput::ParticipationObserved {
    peer_id: 1,
    freshness: 0,
    current_marker: 0,
});
```

### Safety & Invariants

The state machine enforces the following invariants, validated through property-based tests:

- **Exit-at-most-once**: Once exited, no further inputs mutate state.
- **Non-decreasing counts**: Phase 1 and Phase 2 counts never decrease.
- **Quorum implies local completed**: Quorum exit cannot happen before `LocalParticipationCompleted`.
- **Deadline exits**: `DeadlineExpired` transitions to exited state regardless of quorum.
- **Duplicate input idempotence**: Duplicate observations never increase counts.
- **Stale input idempotence**: Stale observations never mutate state.
- **Non-member isolation**: Observations from non-member peers are ignored.

---

## `faction-validation` — Scenario Harness

### Overview

`faction-validation` provides a deterministic test harness for simulating multi-node readiness scenarios. It is useful for integration testing, golden path testing, and property-based validation of cluster readiness coordination.

### Key Types

| Type | Description |
|---|---|
| `ClusterReadinessScenarioHarness` | A multi-coordinator harness that manages a shared `current_marker` timeline. |
| `ClusterReadinessScenarioNode` | A single-node wrapper that follows a consume-and-return pattern for deterministic replay. |
| `ClusterReadinessScenarioEvent` | An event representation for scripted scenario sequences across multiple nodes. |
| `ClusterReadinessScenarioTraceEntry` | A trace entry recording an event, its transition outputs, and the resulting snapshot. |

### Example

```rust
use faction_validation::cluster_readiness_scenario_harness::ClusterReadinessScenarioHarness;

let mut harness = ClusterReadinessScenarioHarness::new(
    vec![0, 1, 2, 3, 4],  // peer set
    4,                      // quorum threshold
    2,                      // max delay margin
);

// Simulate participation observations from all peers
for peer_id in 0..5 {
    harness.apply_participation(0, peer_id, 0);
}

// Complete local participation
let outputs = harness.complete_local_participation(0);
```

---

## Quality Gates

Run the following before committing any changes:

```powershell
# Stage 1 — format, clippy, no_std checks, unit + integration tests
powershell -File scripts\run_stage_1.ps1

# Stage 2 — coverage (crap4rust) + file risk analysis
powershell -File scripts\run_stage_2.ps1
```

Both stages must pass.

---

## Repository Structure

```
.
├── Cargo.toml              # Workspace manifest
├── core/                   # faction crate (state machine)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── cluster_readiness.rs
│   │   ├── cluster_readiness_config.rs
│   │   ├── cluster_readiness_input.rs
│   │   ├── cluster_readiness_observer.rs
│   │   ├── cluster_readiness_output.rs
│   │   ├── cluster_readiness_snapshot.rs
│   │   ├── cluster_readiness_transition.rs
│   │   ├── freshness_classification.rs
│   │   ├── freshness_policy.rs
│   │   ├── no_op_cluster_readiness_observer.rs
│   │   ├── output_batch.rs
│   │   ├── quorum_policy.rs
│   │   ├── readiness_exit_mode.rs
│   │   └── readiness_lifecycle_state.rs
│   └── tests/
│       ├── all_tests.rs
│       ├── cluster_readiness_tests.rs
│       ├── cluster_readiness_observer_tests.rs
│       ├── cluster_readiness_snapshot_tests.rs
│       ├── freshness_policy_tests.rs
│       ├── output_batch_tests.rs
│       └── property_tests/
└── validation/             # faction-validation crate (scenario harness)
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── cluster_readiness_scenario_event.rs
    │   ├── cluster_readiness_scenario_harness.rs
    │   ├── cluster_readiness_scenario_node.rs
    │   └── cluster_readiness_scenario_trace_entry.rs
    └── tests/
        ├── all_tests.rs
        ├── cluster_readiness_validation_exit_tests.rs
        ├── cluster_readiness_validation_harness_tests.rs
        ├── cluster_readiness_validation_participation_tests.rs
        ├── cluster_readiness_validation_pathology_tests.rs
        ├── cluster_readiness_validation_ready_tests.rs
        └── property_tests/
```

---

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](./LICENSE).