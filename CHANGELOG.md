# Changelog

## [Unreleased]

### Self-describing entry point, Architecture Decision Records, and a documentation sweep

#### Added
- Architecture Decision Records under `docs/ADRs/` — nine records documenting the
  core invariants, one property per file: protocol-agnostic, pure-Mealy (no I/O),
  deterministic and replayable, `no_std` + zero-unsafe, single entry point,
  totality via the exhaustive `(state, command)` matrix, stateful-but-persistency-free,
  total observability, and state-as-trait-object.
- The README quick-start is now compiled and executed as a doctest (via
  `include_str!`), so it can no longer drift from the public API.
- The valid transition matrix now asserts the returned `cluster_view` equals a
  subsequent `Probe` for every `(state, command)` case.

#### Changed
- **Breaking** — `ProcessResult::Accepted` now carries an `admissible` field:
  `Accepted { cluster_view, admissible, outcomes }`. All three result variants
  (`Accepted`, `Rejected`, `Probed`) now expose the set of next-admissible
  commands, so a caller can ask "what may I send next?" uniformly regardless of
  outcome.
- **Breaking** — removed the `faction::PeerId` re-export. Import
  `faction::types::PeerId` directly; the alias now lives in exactly one place.
- Documentation sweep across `README.md`, `ARCHITECTURE.md`, and `ROADMAP.md`:
  reconciled test counts (277) and core productive LOC (~885), propagated the
  `admissible`-on-every-variant change, softened the formal-verification claim,
  linked the new ADRs, and corrected the worked examples.
- Curated `OPEN_POINTS.md` down to the genuinely open items — the quorum-change
  question and the deferred ADRs — dropping everything now implemented, fixed,
  or captured in an ADR.
- Split the transition-matrix test helpers into focused `builder` and
  `assertions` modules.
- **Breaking** — removed the `TimedOut` state and `Conclusion::TimedOut`.
  `Bootstrapped` is now the only terminal state; a missed deadline is surfaced
  as `PeerState::TimedOut` (a derived view flag) without concluding. Added
  `Outcome::AcknowledgeRejoin { peer_id }` and
  `Outcome::DeadlineMissed { confirmed_count }`, and `Bootstrapped` now admits
  `ParticipationObserved`.

#### Fixed
- Single-node clusters (`size == 1`) now bootstrap. `Initial` accepted only
  peer-driven input, so a lone node's self-driven `LocalParticipationCompleted`
  was rejected and it never left `Initial`. `Initial` now accepts
  `LocalParticipationCompleted`, which also closes a latent ordering hazard
  where the first node to complete locally — before observing any peer — could
  wedge at any cluster size.
- **The concluded-node silent-sink bug.** A node that
  reached a terminal state stopped helping peers: `Bootstrapped` cancelled its
  retries and went silent, stranding a peer that missed its readiness; and
  `DeadlineExpired` drove the node into a `TimedOut` dead-end it could never
  leave, even when the readies it needed arrived late. Now a bootstrapped node
  re-advertises its readiness to a still-pinging peer (`AcknowledgeRejoin`), and
  a missed deadline is a non-terminal `DeadlineMissed` fact — the node stays
  receptive and still converges.
- Corrected a `protocol-validation` harness bug: `Cluster::step_transport_node`
  attributed a node's transport-triggered outputs to the incoming message's
  sender instead of the processing node.

#### Removed
- Stale `VALIDATION.md` — superseded by the ADRs and the live test suite.

---

## [0.3.3] — 2026-05-09

### Documentation & polish — runnable example, bootstrapping-gap note, import cleanup

#### Added
- Module-level documentation with a runnable example in `lib.rs` — a full
  drive of the machine (participation, duplicate rejection, `Probe`, quorum
  exit) that runs as a doctest.
- README paragraph documenting the IBFT/Raft bootstrapping gap `faction`
  fills, with downstream-integration context.

#### Changed
- Documentation refresh across the crate docs and README.
- Replaced inline module-path qualifiers (`mpsc::`, `oneshot::`, and other
  module qualifiers) with `use` imports.
- `ClusterBuilder::new`'s second parameter renamed `required` → `node_required`
  for clarity — `system-tests` harness only, not part of the published core API.

---

## [0.3.2] — 2026-05-03

### `Observer` no longer requires `Send`

#### Changed
- Removed the `: Send` supertrait bound from `faction::Observer`.
  The bound was unnecessarily strict for `no_std` and single-threaded embedded
  contexts, where thread-safety guarantees are meaningless and preventing
  non-`Send` observers (e.g. `Rc<RefCell<_>>`-backed) from being used.
  Implementations that genuinely need `Send` (e.g. `SharedFileObserver`,
  `NoOpObserver`) remain `Send` — the bound is simply no longer enforced
  at the trait level.

#### Adapted — `system-tests`
- `Node::Task` now holds `Rc<RefCell<FactionNode>>` instead of
  `Arc<Mutex<FactionNode>>` — single-threaded polling never needed the
  overhead of a mutex or the `Send` requirement.
- `Node::Thread` no longer holds a shared reference to `FactionNode`.
  The node is now constructed *inside* the spawned thread closure, which
  keeps the non-`Send` observer local to the thread that owns it.
  The final `PeerState` is communicated back to the polling loop via a
  lightweight `Arc<Mutex<PeerState>>` written once on completion.
- `Node::spawn_thread` signature changed from `(Arc<Mutex<FactionNode>>)`
  to `(impl FnOnce() -> FactionNode + Send + 'static)` — the closure
  carries only `Send` construction data; the node itself is never moved
  across thread boundaries.

---


## [0.3.1] — 2026-05-03

### Post-hardening cleanup — dead outcomes removed, unsafe denied, timer architecture unified

#### Removed
- `StaleParticipationIgnored` and `StaleReadyIgnored` from `Outcome` — these variants were
  never produced by any state. Removed to keep the public API free of dead arms.
- `FreshnessPolicy` and `FreshnessClassification` source files — orphaned files that were
  never compiled (not listed in `lib.rs`) and referenced a non-existent `Freshness` type.
- `InMemoryTimer` from `system-tests` — functionally identical to `RealTimer` after the
  `with_delay` addition; the deterministic zero-delay use case belongs to `protocol-validation`.
- `TimerKind` enum from `system-tests` — redundant once `InMemoryTimer` was removed.
- `--timer` CLI argument from `faction-node` binary — only one timer implementation remains.
- 5 convergence test variants using `InMemoryTimer` (`task_inmemory_inmemory`,
  `task_inmemory_channels`, `thread_inmemory_inmemory`, `thread_inmemory_channels`,
  `thread_inmemory_tcp`) — coverage preserved by the equivalent `real` timer variants.
  Convergence matrix reduced from 15 to 10 variants.

#### Added
- `#![deny(unsafe_code)]` at crate root in all six entry points: `faction` (core),
  `faction-core-validation`, `faction-protocol`, `faction-protocol-validation`,
  `faction-system-tests`, and `faction-node` binary. The "0-unsafe" property is now
  enforced at compile time, not just claimed in documentation.

#### Fixed
- `thread_inmemory_tcp` instability — the `InMemoryTimer` fired at zero delay, flooding
  TCP with `RetryPing`/`RetryReady` broadcasts faster than the transport could drain them.
  Root cause eliminated by removing the combination entirely.

---

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
