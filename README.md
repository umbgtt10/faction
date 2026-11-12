# faction

**A `no_std`, 0-unsafe Mealy machine for distributed cluster bootstrapping.**

Every distributed system needs to answer one question before it can do anything else:
*"Is the cluster ready to proceed?"*

Most systems answer it with ad-hoc coordination, custom timeouts, heuristics,
magic numbers tuned by intuition, and startup sequences that were never tested in
isolation. When they break, they break silently, under load, in production.

Consensus protocols don't help. IBFT, PBFT, and Tendermint assume the cluster
already exists — they consume a static validator set and answer "what's the next
block?" without ever asking "are we a cluster yet?" Raft tries harder but fuses
bootstrapping directly into leader election and log replication. When a Raft
cluster won't form, you face a single opaque failure mode indistinguishable from
a network partition, a slow peer, or a config mismatch. Both families of protocol
skip the same gate: is the set of peers actually alive and in agreement before
consensus begins?

`faction` replaces that with a **formally specified, fully tested Mealy state machine**
that answers the question deterministically, observably, and provably.

---

## What faction does

`faction` tracks participation and readiness signals across a known set of peers and
emits a deterministic exit decision — **Bootstrapped** — when quorum forms. A missed
deadline is reported as **TimedOut**, but is not terminal: the node stays receptive.

It does exactly this. Nothing more.

No network I/O. No consensus algorithm. No opinion on what transport you use, what
"ready" means, or how your protocol works. The caller owns all of that. `faction` owns
the state transitions.

---

## Why faction?

**The problem with ad-hoc bootstrapping:**

Bootstrapping logic is typically written once, tested never, and debugged in production.
It interacts with failure detection, membership management, and consensus in ways that
are hard to reason about in isolation. The bugs it hides surface under Byzantine
conditions — partial startup, network partitions, delayed nodes — that are difficult to
reproduce and expensive to diagnose.

**What faction provides instead:**

- **Deterministic** — `output = F(state, input)`. Same inputs always produce the same
  outputs. Any execution is replayable from its input log.
- **Verifiable** — every `(state, command)` pair is explicitly tested. No untested
  paths exist. The test suite is a proof, not a sample.
- **Embeddable** — `no_std + alloc`, zero `unsafe`. Runs on bare metal, WASM, embedded
  RTOS, and cloud without modification.
- **Observable** — every call reaches a trait-based `Observer`, once, with the command
  and its result. Wire it to telemetry, audit logs, or test assertions. No
  instrumentation surprises.
- **Queryable** — probe the machine at any time for the current cluster view and the
  set of admissible commands. Zero side effects.
- **Slim by construction** — each state carries only its active data. The terminal
  state carries no heap allocation beyond what it received.

---

## How it works

### The state machine

The machine progresses through four states, with a single terminal:

```text
Initial → Pinging → Collecting → Bootstrapped
```

| State | Meaning | Carries |
|---|---|---|
| `Initial` | Freshly created, no action taken | Nothing — unit struct |
| `Pinging` | Collecting participation signals from peers | In-flight participation and readiness sets |
| `Collecting` | Local participation complete, collecting readiness | In-flight readiness and completed participation sets |
| `Bootstrapped` | Quorum reached — cluster is ready (terminal) | Final peer sets at time of exit |

A missed deadline is **not** a state: it is recorded as a fact (reported as
`PeerState::TimedOut` through a derived flag) while the node stays receptive and
can still converge. And `Bootstrapped` is not a silent sink — it re-advertises its
readiness to a peer that is still trying to join.

### Commands and outcomes

Eight commands drive the machine:

| Command | Meaning |
|---|---|
| `ParticipationObserved { peer_id }` | A peer signalled participation |
| `ReadyObserved { peer_id }` | A peer signalled readiness |
| `LocalParticipationCompleted` | The local node finished its own participation |
| `DeadlineExpired` | Deadline timer fired |
| `JoinRequested { peer_id }` | A newcomer asked to join |
| `JoinApproved { peer_id }` | A newcomer was admitted to membership |
| `JoinRejected { peer_id }` | A newcomer was denied |
| `Probe` | Query current state without mutation |

Every command produces a structured result — `Accepted`, `Rejected`, or `Probed` —
each carrying the current cluster view and the set of commands admissible next (plus
the outcomes, on `Accepted`). The machine never panics, never returns an opaque error,
and never silently ignores input.

### Two-phase design

The machine enforces a deliberate two-phase protocol:

**Phase 1 — Pinging:** nodes observe each other's participation signals. A node stays
in this phase until it has signalled its own participation locally.

**Phase 2 — Collecting:** the node collects readiness signals from peers. Quorum is
only checked in this phase — preventing premature exit before local participation
is complete.

This separation eliminates an entire class of race conditions where a node could
declare quorum before confirming its own participation.

### Dynamic joining

Membership is not frozen at genesis. A newcomer can join a running cluster: the
`JoinRequested` / `JoinApproved` / `JoinRejected` commands drive a membership axis
orthogonal to the bootstrap states, so a node in **any** state — including terminal
`Bootstrapped` and non-terminal `TimedOut` — can admit or deny a newcomer and keep
converging. A sub-quorum cluster that timed out on its own recovers the moment a
newcomer supplies the missing member. Admission is a local control-plane decision,
not a wire message, so no transport has to learn a new message type.

### Validation harness

`faction` ships with a multi-layered validation harness:

| Harness | What it tests |
|---|---|
| [`spec/`](./spec/SPEC.md) | The normative specification: a decision table of 18 rows and 41 conformance vectors |
| `spec-model/` | The specification's independent model: its decision table read and interpreted, depending on nothing of the crate — 64 tests |
| `core/` unit tests | Each type and each step in isolation — 128 tests |
| `ladder/` | Conformance to the specification, of the model and then of the crate; every `(state, command)` pair, checked against the decision table; properties over random command sequences, the crate against the model among them; logs replayed — 85 tests |
| `core-validation/` | Multi-node deterministic scenarios — 39 tests |
| `protocol/` | Message translation and protocol runtime — 42 tests |
| `protocol-validation/` | Convergence sweeps across cluster/quorum sizes — 169 tests |
| `system-tests/` convergence | Bootstrapping across spawn/transport — 20 tests |
| `system-tests/` joining | Dynamic joining across spawn/transport — 61 tests |
| `system-tests/` infrastructure | TCP, gRPC, channels, in-memory plumbing, timers, the process node and the harness itself — 155 tests |

The system test matrix covers every valid combination of spawn model and transport:

| Spawn | Transport | Timer |
|---|---|---|
| Task | In-memory, Channels, TCP, gRPC | Real, In-memory |
| Thread | In-memory, Channels, TCP, gRPC | Real, In-memory |
| Process | TCP, gRPC | Real |

These are not simulated tests. Process-based cases spawn real OS processes communicating
over real TCP and gRPC connections. The machine's correctness is verified end-to-end
across all combinations.

---

## Quick start

```toml
[dependencies]
faction = "0.3"
```

```rust
use faction::command::Command;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::peer_state::PeerState;
use faction::process_result::ProcessResult;
use faction::quorum_policy::QuorumPolicy;

extern crate alloc;

let config = Config::new(
    0,
    alloc::vec![0, 1, 2, 3, 4],
    QuorumPolicy::new(4),
);

let mut machine = Faction::new(config, Box::new(NoOpObserver));

machine.process(Command::ParticipationObserved { peer_id: 1 });
machine.process(Command::ParticipationObserved { peer_id: 2 });
machine.process(Command::LocalParticipationCompleted);
machine.process(Command::ReadyObserved { peer_id: 1 });
machine.process(Command::ReadyObserved { peer_id: 2 });

// Self + peers 1, 2, 3 = 4 confirmations → quorum → Bootstrapped.
let result = machine.process(Command::ReadyObserved { peer_id: 3 });
if let ProcessResult::Accepted { cluster_view, admissible, .. } = result {
    assert_eq!(cluster_view.peer_state(), PeerState::Bootstrapped);
    // Concluded, but not a silent sink: a bootstrapped node still admits
    // `ParticipationObserved` (to re-advertise readiness to a still-joining
    // peer) plus the Phase 1 join commands.
    assert_eq!(
        admissible,
        alloc::vec![
            Command::ParticipationObserved { peer_id: 0 },
            Command::JoinRequested { peer_id: 0 },
            Command::JoinApproved { peer_id: 0 },
            Command::JoinRejected { peer_id: 0 },
            Command::Probe
        ]
    );
}
```

The caller owns the network. `faction` owns the state.

---

## Project status

| Metric | Value |
|---|---|
| Core productive LOC | ~1,130 |
| Specification | [`spec/`](./spec/SPEC.md) 0.1.0 — its independent model and the crate answer all 41 vectors as written |
| Total tests | 926 |
| Code coverage (productive) | 100% |
| `(state, command)` matrix | [state_transition_matrix_tests.rs](./ladder/tests/matrix/state_transition_matrix_tests.rs) |
| Crappy functions (CRAP score) | 0 |
| Unsafe code | 0 — enforced by `#![forbid(unsafe_code)]` |
| `no_std` | Verified |
| System test combinations | 10 (Task/Thread/Process × Memory/Channels/TCP/gRPC) |
| Published | [crates.io](https://crates.io/crates/faction) |

---

## Design principles

- **Pure Mealy** — `output = F(state, input)`. No side effects inside the machine.
  The machine computes. The caller acts.
- **Explicit state ownership** — states carry only what they mutate. Nothing is
  inherited silently.
- **No silent sinks** — the sole terminal state (`Bootstrapped`) still answers a
  still-pinging peer by re-advertising its readiness, and a missed deadline is a
  non-terminal fact — so a concluded node never strands the peers it coordinated.
- **Observer, not logger** — the `Observer` trait receives every call with its result:
  an accepted command, a rejected one, a probe. Wire it to anything. The machine does
  not care.
- **Protocol-agnostic** — `faction` does not know what a peer is, what the network
  looks like, or what your protocol does. It knows state transitions.
- **One struct per file** — each step, state, and policy is its own file. Navigation
  is O(1).
- **No `&mut` parameters** — prefer return values over in-place mutation.
- **Persistency-free** — the machine is stateful but never owns its state durably.
  Nothing is written to disk; state is rebuilt by replaying the input log. Durability
  is the caller's.

Each principle above is recorded as an Architecture Decision Record — see
[docs/ADRs/](./docs/ADRs/) for the authoritative rationale.

---

## Non-goals

`faction` deliberately does not:

- Perform network I/O — the caller sends and receives messages
- Implement failure detection — that is Phase 2
- Manage the full membership lifecycle — Phase 1 joining has landed; add/remove and reconfiguration are Phases 2–6
- Know about consensus — protocols build on top of `faction`, not inside it
- Provide a runtime — async, threading, and process management are the caller's concern

---

## Quality gates

```sh
just stage1   # format, clippy, no_std check, the tests, then the system tests
just stage2   # house rules, duplication, CRAP, coverage, mirrored tests, file risk, dependencies
```

Both gates must pass before any commit lands. They drive these tools, each installed
once from crates.io:

| Tool | Used by | Install |
|---|---|---|
| [`just`](https://crates.io/crates/just) | Runs the two stages | `cargo install just` |
| [`slotgate`](https://crates.io/crates/slotgate) | Stage 1 runs the system tests as bounded-parallel per-test processes (a disjoint port range per slot) instead of a serial `--test-threads=1` pass | `cargo install slotgate` |
| [`cargo-stern4rust`](https://crates.io/crates/cargo-stern4rust) | Stage 2 checks the house coding rules in every package | `cargo install cargo-stern4rust` |
| [`cargo-dry4rust`](https://crates.io/crates/cargo-dry4rust) | Stage 2 refuses duplicated code in `core` and `protocol` | `cargo install cargo-dry4rust` |
| [`cargo-crap4rust`](https://crates.io/crates/cargo-crap4rust) | Stage 2 scores CRAP (Change Risk Anti-Patterns) complexity over `core` and `protocol` | `cargo install cargo-crap4rust` |
| [`cargo-llvm-cov`](https://crates.io/crates/cargo-llvm-cov) | Stage 2 requires every line, region and function of `core` and `protocol` covered | `cargo install cargo-llvm-cov` |
| [`cargo-twin4rust`](https://crates.io/crates/cargo-twin4rust) | Stage 2 requires a mirrored test file for every source file of `core` and `protocol` | `cargo install cargo-twin4rust` |
| [`cargo-iceberg4rust`](https://crates.io/crates/cargo-iceberg4rust) | Stage 2 bounds each file's private implementation risk | `cargo install cargo-iceberg4rust` |
| [`cargo-deny`](https://crates.io/crates/cargo-deny) | Stage 2 checks advisories, licences and sources | `cargo install --locked cargo-deny` |

---

## Roadmap

`faction` is building toward full dynamic membership across six incremental phases.
**Phase 1 (dynamic joining)** has landed — a newcomer joins a running cluster and
converges across the full spawn/transport matrix. Failure detection, single-node
addition and removal, and Byzantine-tolerant reconfiguration follow in Phases 2–6.

Each phase is a strict superset of the previous; Phase 0 tests pass unchanged at Phase 6.
No phase begins until the previous phase has 100% `(state, command)` coverage.

See [ROADMAP.md](./docs/ROADMAP.md) for the full plan and [ARCHITECTURE.md](./docs/ARCHITECTURE.md)
for the complete technical specification.

---

## Workspace

| Crate | Role | Tests |
|---|---|---|
| `core/` | State machine | 128 |
| `spec-model/` | The specification's independent model, `no_std` | 64 |
| `ladder/` | Conformance, the decision-table matrix, property and replay tiers | 85 |
| `core-validation/` | Deterministic multi-node scenario harness | 39 |
| `protocol/` | Message translator and protocol runtime | 42 |
| `protocol-validation/` | Convergence sweeps + in-process protocol cluster | 169 |
| `system-tests/` convergence | Bootstrapping across spawn/transport | 20 |
| `system-tests/` joining | Dynamic joining across spawn/transport | 61 |
| `system-tests/` infrastructure | TCP, gRPC, channels, in-memory plumbing, timers, the process node and the harness itself | 155 |
| `xtask/` | The gates: `cargo xtask stage2`, the system tests, the commit scores | 163 |

---

## License

MIT. See [LICENSE](./LICENSE).

---

## Links

- [SPEC.md](./spec/SPEC.md) — the normative specification: data model, decision table, prose and conformance vectors
- [ARCHITECTURE.md](./docs/ARCHITECTURE.md) — the design, and how the crate implements the specification
- [ADRs](./docs/ADRs/) — architecture decision records: the design rationale, one property per file
- [ROADMAP.md](./docs/ROADMAP.md) — phased development plan (Phases 1–6)
- [OPEN_POINTS.md](./docs/OPEN_POINTS.md) — open design questions (quorum change, `PeerId` genericization)
- [CHANGELOG.md](./CHANGELOG.md) — version history
- [ETHEREUM.md](./docs/ETHEREUM.md) — why `faction` matters to the Ethereum ecosystem
- [DONATE.md](./DONATE.md) — support the project
