# Changelog

## [0.2.0] — 2025-04-26

### Phase 0 complete — Hardened machine

The crate has been fully hardened through adversarial testing, complete matrix coverage, and a structural redesign that eliminates dead code and clarifies state ownership.

#### Added
- `StateSnapshot` trait — each state returns a delta over the previous snapshot.
  States own only their active data; frozen fields are inherited automatically.
- `MachineSnapshot` builder methods (`.with_lifecycle_state()`, `.with_exit_mode()`,
  `.with_local_participation_complete()`, `.with_readiness_exited()`,
  `.with_phase1_count()`, `.with_phase2_count()`).
- `ClusterSimulation` — multi-node broadcast bus for convergence testing.
  Automatically translates `BroadcastLocalReady` into `ReadyObserved` for all peers.
- 10 observer tests covering all transition paths where `step()` is called.
- 6 `MachineSnapshot` builder unit tests — each verifies only the target field changes.
- 6 per-state `state_snapshot` inheritance tests.
- 3 convergence tests (quorum, deadline fallback, duplicate resilience).
- `CHANGELOG.md`.

#### Changed
- **Renamed `Vibe` → `Machine`** across the entire workspace (50+ files).
  - `VibeConfig` → `MachineConfig`, `VibeInput` → `MachineInput`,
    `VibeOutput` → `MachineOutput`, `VibeSnapshot` → `MachineSnapshot`,
    `VibeState` → `MachineState`, `VibeTransition` → `MachineTransition`,
    `VibeObserver` → `MachineObserver`, `NoOpVibeObserver` → `NoOpMachineObserver`.
- `deal()` → `accept()`, `punch()` → `step()`, `vibe_check()` → `snapshot()`.
- `MachineState` no longer has a standalone `snapshot()` method — it extends `StateSnapshot` instead.
- `Machine::apply()` composes the full snapshot from cached previous + delta.
- `Collecting` dropped `phase1: ConfirmedSet` — carries only `phase2: ConfirmedSet`
  and a frozen `phase1_count: usize`.
- `ReadyByQuorum` and `ReadyByDeadline` dropped `ConfirmedSet` fields — carry
  only frozen `phase1_count` and `phase2_count` (both `usize`).
- `ReadyByDeadline` dropped `local_participation_complete` — inherited from
  previous snapshot (correct for both Pinging and Collecting entry paths).
- Terminal state `step()` replaced with `unreachable!()` — gated by `accept()` returning `false`.
- Default `accept()` → `true` on the trait. `Pinging` removed its override.
- `VibeScenarioEvent` → `MachineScenarioEvent`, `VibeScenarioNode` → `MachineScenarioNode`,
  `VibeScenarioTraceEntry` → `MachineScenarioTraceEntry`.
- Bumped version to `0.2.0`.

#### Removed
- Dead `punch` match arms in `Collecting`, `ReadyByQuorum`, `ReadyByDeadline`.
- Temporary `Box::new(Initial)` allocation in `Vibe::apply()` — uses `Option<Box<...>>` with `take()`.
- `pub(super)` visibility on all state struct fields (all `pub` now).

---

## [0.1.0] — Initial release

- Two-phase cluster readiness state machine.
- Five states: `Initial`, `Pinging`, `Collecting`, `ReadyByQuorum`, `ReadyByDeadline`.
- Freshness classification (timely, delayed-within-margin, stale).
- Quorum-based and deadline-based exit.
- `NoOpVibeObserver` and `MachineObserver` trait.
- `faction-validation` crate with `VibeScenarioHarness` for multi-node scenario testing.
- Property-based invariants via `proptest`.
- `ConfirmedSet` helper with pure functional `try_confirm()` / `confirm()`.
- `compute_output` helper for deterministic output computation.
- 20 `(state, input)` matrix tests via `rstest`.
- Per-state integration tests for all five states.
- Core: 139 tests. Validation: 29 tests.