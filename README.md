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
emits a deterministic exit decision — either **Bootstrapped** or **TimedOut**.

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
- **Observable** — every transition reaches a trait-based `Observer`. Wire it to
  telemetry, audit logs, or test assertions. No instrumentation surprises.
- **Queryable** — probe the machine at any time for the current cluster view and the
  set of admissible commands. Zero side effects.
- **Slim by construction** — each state carries only its active data. Terminal states
  carry no heap allocation beyond what they received.

---

## How it works

### The state machine

The machine progresses through five states:

```
Initial → Pinging → Collecting → Bootstrapped
                         ↓
                      TimedOut
```

| State | Meaning | Carries |
|---|---|---|
| `Initial` | Freshly created, no action taken | Nothing — unit struct |
| `Pinging` | Collecting participation signals from peers | In-flight participation and readiness sets |
| `Collecting` | Local participation complete, collecting readiness | In-flight readiness and completed participation sets |
| `Bootstrapped` | Quorum reached — cluster is ready (terminal) | Final peer sets at time of exit |
| `TimedOut` | Deadline expired before quorum (terminal) | Peer sets at time of expiry |

### Commands and outcomes

Five commands drive the machine:

| Command | Meaning |
|---|---|
| `ParticipationObserved { peer_id }` | A peer signalled participation |
| `ReadyObserved { peer_id }` | A peer signalled readiness |
| `LocalParticipationCompleted` | The local node finished its own participation |
| `DeadlineExpired` | External deadline timer fired |
| `Probe` | Query current state without mutation |

Every command produces a structured result — `Accepted`, `Rejected`, or `Probed` —
with full outcome detail and an updated cluster view. The machine never panics, never
returns an opaque error, and never silently ignores input.

### Two-phase design

The machine enforces a deliberate two-phase protocol:

**Phase 1 — Pinging:** nodes observe each other's participation signals. A node stays
in this phase until it has signalled its own participation locally.

**Phase 2 — Collecting:** the node collects readiness signals from peers. Quorum is
only checked in this phase — preventing premature exit before local participation
is complete.

This separation eliminates an entire class of race conditions where a node could
declare quorum before confirming its own participation.

### Validation harness

`faction` ships with a multi-layered validation harness:

| Harness | What it tests |
|---|---|
| `core/` unit tests | Every `(state, command)` pair — 145 tests |
| `core-validation/` | Multi-node deterministic scenarios — 23 tests |
| `protocol/` | Message translation and protocol runtime — 33 tests |
| `protocol-validation/` | In-process protocol cluster — 9 tests |
| `system-tests/` | Multi-process convergence across all spawn/transport combinations — 54 tests |

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
machine.process(Command::ReadyObserved { peer_id: 3 });

// Quorum of 4 reached → Bootstrapped.
let result = machine.process(Command::ReadyObserved { peer_id: 4 });
if let ProcessResult::Accepted { cluster_view, .. } = result {
    assert!(cluster_view.is_concluded());
}
```

The caller owns the network. `faction` owns the state.

---

## Project status

| Metric | Value |
|---|---|
| Productive LOC | 1,165 |
| Total tests | 264 |
| Code coverage (productive) | 100% |
| `(state, command)` matrix | [transition_matrix_tests.rs](./core/tests/transition_matrix/state_transition_matrix_tests.rs) |
| Crappy functions (CRAP score) | 0 |
| Unsafe code | 0 — enforced by `#![deny(unsafe_code)]` |
| `no_std` | Verified |
| System test combinations | 15 (Task/Thread/Process × Memory/Channels/TCP/gRPC) |
| Published | [crates.io](https://crates.io/crates/faction) |

---

## Design principles

- **Pure Mealy** — `output = F(state, input)`. No side effects inside the machine.
  The machine computes. The caller acts.
- **Explicit state ownership** — states carry only what they mutate. Nothing is
  inherited silently.
- **No dead code** — terminal states return `false` from `accept()`, making `step()`
  structurally unreachable. The compiler enforces this.
- **Observer, not logger** — the `Observer` trait receives every transition, query,
  and rejection. Wire it to anything. The machine does not care.
- **Protocol-agnostic** — `faction` does not know what a peer is, what the network
  looks like, or what your protocol does. It knows state transitions.
- **One struct per file** — each step, state, and policy is its own file. Navigation
  is O(1).
- **No `&mut` parameters** — prefer return values over in-place mutation.

---

## Non-goals

`faction` deliberately does not:

- Perform network I/O — the caller sends and receives messages
- Implement failure detection — that is Phase 2
- Manage dynamic membership — that is Phases 1–6
- Know about consensus — protocols build on top of `faction`, not inside it
- Provide a runtime — async, threading, and process management are the caller's concern

---

## Quality gates

```powershell
powershell -File scripts\run_stage_1.ps1   # format, clippy, no_std checks, full test suite
powershell -File scripts\run_stage_2.ps1   # CRAP score and file risk analysis
```

Both gates must pass before any commit lands.

---

## Roadmap

`faction` is building toward full dynamic membership — node joining, failure detection,
single-node addition and removal, and Byzantine-tolerant reconfiguration across six
incremental phases.

Each phase is a strict superset of the previous; Phase 0 tests pass unchanged at Phase 6.
No phase begins until the previous phase has 100% `(state, command)` coverage.

See [ROADMAP.md](./docs/ROADMAP.md) for the full plan and [ARCHITECTURE.md](./docs/ARCHITECTURE.md)
for the complete technical specification.

---

## Workspace

| Crate | Role | Tests |
|---|---|---|
| `core/` | State machine — 13 source files | 145 |
| `core-validation/` | Deterministic multi-node scenario harness | 23 |
| `protocol/` | Message translator and protocol runtime | 33 |
| `protocol-validation/` | In-process protocol cluster | 9 |
| `system-tests/` | Multi-process convergence across all combinations | 54 |

---

## License

MIT. See [LICENSE](./LICENSE).

---

## Links

- [ARCHITECTURE.md](./docs/ARCHITECTURE.md) — complete technical specification
- [ROADMAP.md](./docs/ROADMAP.md) — phased development plan
- [CHANGELOG.md](./CHANGELOG.md) — version history
- [DONATE.md](./DONATE.md) — support the project
