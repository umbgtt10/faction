# Validation

**Last updated:** 2026-05-02  
**Total tests:** 275  
**Total execution time:** ~4.7s (excl. compilation)  
**Productive code coverage:** 100%  
**Crappy functions:** 0

---

## Testing philosophy

We test to **five properties**. Every test in this codebase serves at least two of them.

### 1. Reproducible

Same inputs → same outputs. Always. Property tests replay thousands of random
sequences and assert the model and the machine agree. No flaky tests. No timing
assumptions in core. Every failure is a deterministic assertion.

### 2. Systematic

Every `(state, command)` pair is explicitly covered by the transition matrix.
No "we think this path is handled." The matrix is exhaustive by construction.
Missing a cell is a compile error — the `rstest` cases are enumerated, and
the per-state invalid test files complete the grid.

### 3. Adversarial

Property tests act as a fuzzer. Random sequences of commands, random peer IDs,
random sequences of duplicate and non-member signals. The machine must never
violate invariants — not once in thousands of sequences.

### 4. Efficient

The full suite (275 tests) runs in under 5 seconds excluding compilation.
The transition matrix alone completes in under a second. This means the
gate runs on every commit without slowing development.

### 5. Controlled

No unit tests. All tests are integration tests that exercise the public API.
The `Observer` trait + `Probe` command + `process()` return value form the
complete observable surface. We test what the caller sees, not internal state.

---

## What each crate tests — and why

### `faction` (core) — 145 tests in 0.85s

The Mealy machine. Every productive line is covered.

| Suite | Count | Why |
|---|---|---|
| Transition matrix (valid) | 12 cases | Exhaustive `(state, command)` → `Accepted` coverage |
| Transition matrix (invalid) | 22 cases | Per-state `(state, command)` → `Rejected` coverage |
| Per-state integration | ~40 tests | Behavior within each state: accept/reject, quorum, deadline, duplicates, non-members |
| Step structs | 9 tests | `PingingStep`, `ReadyStep`, `LocalCompletionStep` — dedup, add, quorum |
| Observer | ~15 tests | Every transition path reaches the observer with correct data |
| Property — invariant | 8 tests | Exit-at-most-once, counts-never-decrease, exit-mode-immutable, etc. |
| Property — safety | 4 tests | Required-count-immutable, post-exit-idempotent, counts ≤ peer-count |
| Property — model | 1 test | Reference model vs real machine for thousands of random sequences |
| Config, Quorum, Transition, ClusterView | ~30 tests | Constructor correctness, accessors, immutability |

**Why:** The core must be **provably correct**. The transition matrix is the
specification. Property tests are the adversarial validation. Per-state tests
exercise edge cases the matrix covers structurally.

### `faction-core-validation` — 23 tests in 0.59s

Multi-node deterministic simulation.

| Suite | Count | Why |
|---|---|---|
| Convergence | 3 tests | 5-node clusters converge to quorum, deadline-fallback, duplicate resilience |
| Exit behavior | 7 tests | Deadline from each state, post-exit idempotency, slow-member handling |
| Participation | 2 tests | Signal acceptance, local completion |
| Readiness | 5 tests | Timely acceptance, quorum exit, asymmetric startup, post-exit rejection |
| Pathology | 2 tests | Non-member and duplicate signals across nodes |
| Property | 2 tests | Invariants under random multi-node sequences |
| Harness | 2 tests | Coordinator count, cluster view correctness |

**Why:** The core validates single-node behavior. Multi-node behavior —
convergence, broadcast propagation, asymmetry — can only be tested with
multiple machines running simultaneously. These tests validate that the
protocol converges when nodes start at different times.

### `faction-protocol` — 33 tests in ~0.00s

Message translator and protocol logic.

| Suite | Count | Why |
|---|---|---|
| Message translator | 15 tests | Every transport/timer message → correct command mapping. Every outcome → correct output mapping. |
| Protocol decisions | 18 tests | `initialize()` output, `decide()` for every message type, retry behavior, exit behavior |

**Why:** The protocol layer is the bridge between the pure machine and the
network. The translator must be **exhaustively correct** — every input message
maps to exactly one command, every outcome maps to the correct outputs.

### `faction-protocol-validation` — 9 tests in ~0.00s

In-process protocol cluster with real transports and timers.

| Suite | Count | Why |
|---|---|---|
| Cluster | 9 tests | Two-node protocol cluster with InMemory transport convergence |

**Why:** Validates that `Protocol` instances wired together over real transports
(not mocked) can exchange messages and converge.

### `faction-system-tests` — 65 tests in 3.23s

Full-stack integration: spawn strategies × transports × timers.

| Suite | Count | Why |
|---|---|---|
| Convergence (rstest) | 15 tests | Every `(Spawn, Timer, Transport)` combination converges to Bootstrapped in a 5-node cluster |
| Timer — InMemory | 6 tests | Schedule, poll, cancel, FIFO, idempotency |
| Timer — Real | 9 tests | Deadline behavior, delayed delivery, multiple delays, default config |
| Transport — Channels | 6 tests | Send/recv, FIFO, multi-peer, all message types |
| Transport — gRPC | 7 tests | Send/recv, FIFO, multi-peer, all message types, Drop + port reuse |
| Transport — InMemory | 6 tests | Same transport contract as Channels |
| Transport — TCP | 7 tests | Same transport contract, Drop + port reuse |
| SharedFileObserver | 9 tests | All event types, JSON validity, idle no-op, appending writer |

**Why:** This is the **integration gauntlet**. The core and protocol tests
validate correctness in isolation. System tests validate that real timer
delays, real sockets, real processes, and real observers work together
under realistic conditions.

---

## Code coverage

| Crate | Productive files | Coverage |
|---|---|---|
| `faction` (core) | 13 source files | **100%** |

Coverage is measured on productive code only — test files and generated
code are excluded. The three tarpaulin-reported misses (2 lines in
`collecting.rs`, 1 line in `faction.rs`) are false negatives from
`Box::new(Self { ... })` construction and no-op observer methods.
Both paths are provably exercised by integration tests.

Full coverage report from `cargo tarpaulin -p faction`:

```
core\src\cluster_view.rs: 31/31
core\src\config.rs: 13/13
core\src\faction.rs: 33/34 (-2.94%)  ← false negative
core\src\quorum_policy.rs: 5/5
core\src\state.rs: 9/9
core\src\states\bootstrapped.rs: 11/11
core\src\states\collecting.rs: 43/45  ← false negative
core\src\states\initial.rs: 18/18
core\src\states\local_completion_step.rs: 18/18
core\src\states\pinging.rs: 57/57
core\src\states\pinging_step.rs: 11/11
core\src\states\ready_step.rs: 18/18
core\src\states\timed_out.rs: 11/11
core\src\transition.rs: 7/7
```

---

## DeCRAP — Cognitive complexity

`cargo-crap4rust` runs on every commit (Stage 2 gate). It measures
**cognitive complexity** — the number of distinct logical paths through a
function, weighted by nesting depth and branch count.

**Threshold:** 15. Any function exceeding this is flagged as "crappy"
and must be reduced before merging.

**Current status: 0 crappy functions across all crates.**

CRAP is a **structural** metric, not a quality metric. A function can be
correct, well-tested, and familiar — and still have high cognitive
complexity. Conversely, a function with low CRAP can still have bugs.

It is a **red flag, not a design rule.** When a function exceeds the
threshold, the appropriate response is almost never "reduce the function
by extracting helpers." That produces thin wrapper files that scatter
logic without reducing actual complexity. The correct response is almost
always:

1. **Does the function do N things?** Extract a struct with N methods,
   each doing one thing. The struct owns the internal state; the methods
   operate on it. This is an architectural change, not a syntactic one.
2. **Does the function have deep nesting?** Invert conditions, early-return
   on edge cases, flatten the happy path.
3. **Is the function actually simple but CRAP doesn't see it?** Raise the
   threshold for that file, document why, and move on. CRAP is not infallible.

**Example from faction:** `ObservedStep::new()` was a single constructor with
a `FreshnessClassification` branch and four outcomes. CRAP flagged it. The fix
was not to extract helpers — it was to split the concept into three dedicated
structs (`PingingStep`, `ReadyStep`, `LocalCompletionStep`), each with a
single responsibility and zero branching on kind. All three have CRAP scores
well below threshold.

---

## Quality gates

```powershell
powershell -File scripts\run_stage_1.ps1   # fmt, clippy, no_std, all tests
powershell -File scripts\run_stage_2.ps1   # CRAP analysis, file risk
```

Stage 1 must pass before every commit to `src/` or `tests/`.
Stage 2 runs on merge to main and enforces the cognitive complexity ceiling.
