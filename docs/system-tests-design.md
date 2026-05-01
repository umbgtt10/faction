# System tests — Design

**Status:** Draft  
**Crate:** `system-tests`  
**Depends on:** `faction` (core)

---

## Purpose

Validate that N independently running `Faction` instances, communicating over real
transports, converge to `Bootstrapped`. These tests complement the transition matrix
(exhaustive `(state, input)` coverage) and property tests (invariant verification).
Together they validate the full stack: machine correctness → runtime wrapping →
transport framing → process isolation.

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│              test orchestrator                     │
│  ┌─────────────────────────────────────────────┐ │
│  │ spawn nodes, wire transports, poll, assert   │ │
│  └─────────────────────────────────────────────┘ │
│                       │                           │
│              ┌────────▼────────┐                  │
│              │   Transport     │  ← trait         │
│              │ send / recv     │                  │
│              └────────┬────────┘                  │
│         ┌─────────────┼─────────────┐             │
│         ▼             ▼             ▼             │
│   ┌──────────┐ ┌──────────┐ ┌───────────┐        │
│   │in-memory │ │   TCP    │ │   gRPC    │        │
│   │ channels │ │  sockets │ │  streams  │        │
│   └────┬─────┘ └────┬─────┘ └─────┬─────┘        │
│        │            │             │               │
│   ┌────▼────────────▼─────────────▼────┐          │
│   │         node runtime               │          │
│   │  ┌──────────────────────────────┐  │          │
│   │  │ Faction                      │  │          │
│   │  │ process(command) → outcomes  │  │          │
│   │  │        ↕                     │  │          │
│   │  │ recv()  dispatch  send()     │  │          │
│   │  └──────────────────────────────┘  │          │
│   └────────────────────────────────────┘          │
└──────────────────────────────────────────────────┘
```

Three components never change across variants:

1. **Node runtime** — the Faction wrapper + message dispatch loop. Same code whether it's
   in-process, threaded, or a separate OS process. Only knows `send` and `recv`.

2. **Orchestrator** — spawn, poll, assert. Same test logic for all variants. The transport
   factory and spawn strategy are the only parameterized pieces.

3. **Shared-file observer** — every node writes every transition (accepted, queried, rejected)
   to a single file. The orchestrator dumps the full log on any assertion failure.

---

## Workspace

| Crate | Role | Depends on |
|---|---|---|
| `faction` (core) | Pure Mealy machine, `no_std + alloc` | Nothing |
| `system-tests` | Node runtime, transports, integration tests | `faction` |

The node runtime lives inside `system-tests/src/`. No separate `faction-node` library —
it's test infrastructure, not a public API.

```
system-tests/
├── src/
│   ├── main.rs          ← test orchestrator + rstest entry points
│   ├── node.rs          ← Faction wrapper + message dispatch loop
│   ├── transport.rs     ← Transport trait
│   ├── transport/
│   │   ├── in_memory.rs ← InMemoryTransport (mpsc channels)
│   │   ├── tcp.rs       ← TcpTransport
│   │   └── grpc.rs      ← GrpcTransport
│   ├── observer.rs      ← SharedFileObserver
│   └── spawn.rs         ← thread::spawn vs std::process::Command
└── tests/
    └── convergence.rs   ← the rstest with 5 variants
```

---

## Transport trait

All transports implement a single trait. The node runtime is parameterized over it:

```rust
trait Transport {
    fn send(&mut self, to: PeerId, message: Command);
    fn recv(&mut self) -> Option<(PeerId, Command)>;
}
```

The messages on the wire are `Command` values — the same enum used by `Faction::process()`.
The transport is purely framing: serialize + deliver + deserialize. No routing logic,
no decision-making.

Three implementations:

| Transport | Framing | Use |
|---|---|---|
| `InMemoryTransport` | `mpsc::channel` between node pairs | V5 — deterministic correctness |
| `TcpTransport` | Length-prefixed bincode over `TcpStream` | V3/V4 — socket-level validation |
| `GrpcTransport` | Protobuf over gRPC unary RPC | V1/V2 — external interface validation |

---

## Node runtime

A thin loop around a `Faction` instance. Takes a transport, a shared observer, and a peer
set. The loop:

1. `recv()` from the transport — incoming commands from other nodes
2. `faction.process(command)` — feed to the Mealy machine, get outcomes
3. For each outcome that implies a broadcast (`BroadcastLocalReady`), serialize and `send()`
   to every other peer
4. Write the transition to the shared observer
5. Repeat until `cluster_view().peer_state()` is terminal (`Bootstrapped` or `TimedOut`)

The node runtime has no awareness of threads, processes, or network addresses. It holds a
`Box<dyn Transport>` and a `Faction`. The test orchestrator supplies both.

---

## Spawn strategies

| Strategy | Mechanism | Variants |
|---|---|---|
| `Spawn::Thread` | `std::thread::spawn` | V2, V4, V5 |
| `Spawn::Process` | `std::process::Command` | V1, V3 |

Process-spawned nodes receive their peer ID, peer set, transport endpoint, and observer
file path as command-line arguments. The node binary is compiled once and invoked N times.

Thread-spawned nodes share the same address space. The orchestrator owns all node handles
and can poll `cluster_view()` directly through an `Arc<RwLock<ClusterView>>` without
going through the gRPC interface.

---

## Shared-file observer

All nodes write to the same file via the `Observer` trait. The observer implementation
acquires a file lock per write — writes are atomic at the line level, not block level,
to avoid interleaved JSON. The orchestrator reads the complete log on assertion failure.

```rust
struct SharedFileObserver {
    file: Mutex<BufWriter<File>>,
}

impl Observer for SharedFileObserver {
    fn observe(&self, command: Command, transition: Transition) {
        // write JSON line with node_id, timestamp, command, outcomes, old/new state
    }
    fn observe_query(&self, command: Command, cluster_view: ClusterView) { ... }
    fn observe_rejection(&self, command: Command, cluster_view: ClusterView, admissible: Vec<Command>) { ... }
}
```

Each line includes the node ID so the orchestrator can reconstruct per-node and
cluster-wide timelines.

---

## Test structure

A single rstest with 5 variants:

```rust
#[rstest]
#[case::in_memory_tasks(Variant::in_memory())]
#[case::thread_grpc(Variant::grpc(Spawn::Thread))]
#[case::thread_tcp(Variant::tcp(Spawn::Thread))]
#[case::process_grpc(Variant::grpc(Spawn::Process))]
#[case::process_tcp(Variant::tcp(Spawn::Process))]
fn cluster_reaches_bootstrapped(#[case] variant: Variant) {
    let observer = SharedFileObserver::new(tempfile());
    let cluster = variant.create_cluster(PEER_SET, observer.clone());
    cluster.poll_until_bootstrapped(backoff_strategy());

    assert!(cluster.is_bootstrapped(),
        "cluster failed to bootstrap\nlog:\n{}",
        observer.read_all(),
    );
}
```

The test logic is identical across variants — only the `Variant` value type changes.
`Variant` bundles transport factory + spawn strategy. The cluster owns the gRPC endpoints
of all its nodes and coordinates polling internally.

---

## Polling with backoff

The `Cluster` encapsulates the backoff polling logic:

```rust
struct Cluster {
    endpoints: Vec<PeerEndpoint>,
    observer: SharedFileObserver,
}

impl Cluster {
    fn poll_until_bootstrapped(&self, strategy: BackoffStrategy) {
        let deadline = Instant::now() + strategy.total_timeout();
        let mut delay = strategy.initial_delay();

        loop {
            if self.is_bootstrapped() {
                return;
            }
            if Instant::now() > deadline {
                break;
            }
            sleep(delay);
            delay = (delay * strategy.multiplier()).min(strategy.max_delay());
        }
    }

    fn is_bootstrapped(&self) -> bool {
        self.endpoints.iter().all(|ep| {
            ep.cluster_view().peer_state() == PeerState::Bootstrapped
        })
    }
}
```

Default backoff: 1s initial, 2× multiplier, 5s max, 30s total timeout. Configurable
per test case.

The orchestrator never interacts with individual nodes — it delegates polling and
assertions to the `Cluster`.

---

## CI strategy

| Variant | Schedule | Rationale |
|---|---|---|
| V5 (in-memory) | Every commit | Fast, deterministic, no ports — ~100ms |
| V2/V4 (thread + gRPC/TCP) | Every commit | Real sockets but in-process — ~2s |
| V1/V3 (process + gRPC/TCP) | Pre-merge / nightly | Full isolation, real ports — ~5s |

V1/V3 can be `#[ignore]` on default CI and run on a cron schedule or pre-merge gate.

---

## What this validates

| Layer | Validates | Covered by |
|---|---|---|
| Machine correctness | Every `(state, input)` pair is handled | Transition matrix |
| Invariants | Counts, exit, staleness hold under random sequences | Property tests |
| Runtime wrapping | Faction + dispatch loop runs correctly | V5 |
| Transport framing | gRPC and TCP serialize/deserialize correctly | V2, V4 |
| Process isolation | No shared-memory coupling, clean startup/shutdown | V1, V3 |
