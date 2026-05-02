# Changelog

## [0.3.0] — 2026-05-02

### Phase 0 hardened — Freshness removed, step structs split, transport Drop

#### Removed
- **`FreshnessPolicy`** and `FreshnessClassification` — stale/delayed/timely distinction
  removed. Every signal from a member is accepted; duplicates are deduplicated on peer_id.
  The `freshness` and `current_marker` fields removed from `Command::ParticipationObserved`
  and `Command::ReadyObserved`.
- `ObservedKind` enum — replaced by dedicated step structs.
- `ObservedOutput` wrapper — outcome logic inlined into `ObservedStep`.
- 42 stale/delayed-specific tests (193 → 145 in core).

#### Added
- `PingingStep`, `ReadyStep` (renamed from `CollectingStep`), `LocalCompletionStep`
  — three focused step structs replacing `ObservedStep::new()` and `new_local()`.
- `compute_new_state` private helpers on `Collecting` and `Pinging` — decoupled
  from step structs, taking `(is_quorum, confirmed_peers)`.
- `Drop` implementations for all four transports: `GrpcTransport` (oneshot server
  shutdown), `TcpTransport` (AtomicBool + thread join), `ChannelsTransport`,
  `InMemoryTransport`.
- gRPC transport: replaced `block_on` in `send()` with async `mpsc` channel.
  Process test went from ~15s to ~1s.
- Timer integration tests — 15 tests for `InMemoryTimer` and `RealTimer`.
- Transport integration tests — 22 tests covering all four transports.
- `SharedFileObserver` integration tests — 9 tests with `TempFile` cleanup guard.
- `"decisions"` log field now shows full list, not just count.
- `Config::new()` — 3 args (removed `freshness_policy`).
- `build_clients` in gRPC — no `block_on` on `AsyncMutex`, builds `HashMap` first.
- `spawn_server` extraction — shared between `build_server` and `new_mesh`.

#### Changed
- `Exited` → `Concluded`, `ExitMode` → `Conclusion`.
- `CollectingStep` → `ReadyStep` with required `quorum_threshold: usize` (no `Option`).
- `ReadyObserved` in `Pinging` state inlined — no step struct call.
- `start_decisions()` → `initialize()`.
- Tokio runtime: `worker_threads(2)` → `available_parallelism()`.
- `ObservedStep` → removed entirely.
- `compute_output.rs` → inlined into `observed_step.rs`, then split into three files.

#### Fixed
- Port rebinding race in gRPC `new_mesh`.
- `Collecting::new()` dead code removed.
- Temp file collision in `SharedFileObserver` tests.

---

## [0.2.0] — 2025-04-26

### Phase 0 complete — Hardened machine

The crate has been fully hardened through adversarial testing, complete matrix coverage, and a structural redesign that eliminates dead code and clarifies state ownership.

#### Added
- `StateSnapshot` trait — each state returns a delta over the previous snapshot.
  States own only their active data; frozen fields are inherited automatically.
- `ClusterSimulation` — multi-node broadcast bus for convergence testing.
- 10 observer tests covering all transition paths.
- `CHANGELOG.md`.

#### Changed
- **Renamed `Vibe` → `Machine`** across the entire workspace (50+ files).
- `deal()` → `accept()`, `punch()` → `step()`, `vibe_check()` → `snapshot()`.
- Terminal state `step()` replaced with `unreachable!()` — gated by `accept()` returning `false`.
- Bumped version to `0.2.0`.

#### Removed
- Dead `punch` match arms in `Collecting`, `ReadyByQuorum`, `ReadyByDeadline`.

---

## [0.1.0] — Initial release

- Two-phase cluster readiness state machine.
- Five states: `Initial`, `Pinging`, `Collecting`, `ReadyByQuorum`, `ReadyByDeadline`.
- Freshness classification (timely, delayed-within-margin, stale).
- Quorum-based and deadline-based exit.
- `NoOpVibeObserver` and `MachineObserver` trait.
- `faction-validation` crate with `VibeScenarioHarness`.
- Property-based invariants via `proptest`.
- 139 core tests, 29 validation tests.
