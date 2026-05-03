# System tests — Design

**Status:** Implemented  
**Crate:** `system-tests`  
**Depends on:** `faction` (core), `protocol`

---

## Purpose

Validate that N independently running `Protocol` instances, communicating over real
transports, converge to `Bootstrapped`. These tests complement the transition matrix
(exhaustive `(state, command)` coverage) and property tests (invariant verification).
Together they validate the full stack: machine correctness → runtime wrapping →
transport framing → process isolation.

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│              test orchestrator                   │
│  ┌─────────────────────────────────────────────┐ │
│  │ spawn nodes, wire transports, poll, assert  │ │
│  └─────────────────────────────────────────────┘ │
│                       │                          │
│              ┌────────▼────────┐                 │
│              │   Transport     │  ← trait        │
│              │ send / recv     │                 │
│              └────────┬────────┘                 │
│         ┌─────────────┼─────────────┐            │
│         ▼             ▼             ▼           │
│   ┌──────────┐ ┌──────────┐ ┌───────────┐       │
│   │InMemory  │ │   TCP    │ │   gRPC    │       │
│   │ Channels │ │  sockets │ │  streams  │       │
│   └────┬─────┘ └────┬─────┘ └─────┬─────┘       │
│        │            │             │              │
│   ┌────▼────────────▼─────────────▼────┐        │
│   │         FactionNode                │        │
│   │  protocol.decide() → dispatch()    │        │
│   └────────────────────────────────────┘        │
└──────────────────────────────────────────────────┘
```

Three components never change across variants:

1. **FactionNode** — the Protocol + transport + timer + dispatch loop. Same code
   whether it's a task, thread, or separate OS process.
2. **Orchestrator** (`Cluster`) — spawn, poll, assert. Same logic for all variants.
3. **Shared-file observer** — every node writes every transition to a shared file.
   The orchestrator logs the full trace for debugging.

---

## Transport trait

All transports implement the `Transport` trait from `faction-protocol`:

```rust
pub trait Transport: Send {
    fn send(&mut self, to: PeerId, message: TransportMessage);
    fn recv(&mut self) -> Option<TransportMessage>;
}
```

Four implementations:

| Transport | Mechanism | Use |
|---|---|---|
| `InMemoryTransport` | `Arc<Mutex<VecDeque>>` shared inboxes | Fastest — deterministic correctness |
| `ChannelsTransport` | `mpsc::channel` between node pairs | Thread-safe FIFO |
| `TcpTransport` | Length-prefixed frames over `TcpStream` | Socket-level validation |
| `GrpcTransport` | Protobuf over gRPC unary RPC | External interface validation |

---

## FactionNode

A stateful loop around a `Protocol` instance. The loop:

1. Alternates between `timer.poll()` and `transport.recv()`
2. Feeds each event to `protocol.decide()`
3. Dispatches decisions: broadcast, schedule timer, cancel timer
4. Writes state transitions to the observer
5. Stops when `peer_state()` is terminal

The node runtime has no awareness of threads, processes, or network addresses.

---

## Spawn strategies

| Strategy | Mechanism | Transports |
|---|---|---|
| `Spawn::Task` | Single-threaded, cooperative | All (in-memory) |
| `Spawn::Thread` | `std::thread::spawn` | All |
| `Spawn::Process` | `std::process::Command` | TCP, gRPC |

Process-spawned nodes are compiled as `faction-node` binary and invoked with
peer ID, peer set, transport config, timer config, and log path as CLI arguments.

---

## Timer implementations

| Timer | Mechanism | Use |
|---|---|---|
| `RealTimer` | `BinaryHeap<Instant>` — wall-clock deadlines with configurable delay | Real-time simulation |

---

## Shared-file observer

All nodes write to the same file via `SharedFileObserver`. Each line is JSON with
timestamp, peer ID, event type, command/input, and cluster view state. The decisions
field shows the full list of output messages, not just a count.

The orchestrator reads the log on assertion failure for debugging.

---

## Test structure

A single rstest with **10 variants** spanning spawn strategy × transport:

```
# 4 task variants  (Minimal timer delay)
task_real_inmemory    task_real_channels    task_real_tcp    task_real_grpc

# 4 thread variants (Moderate timer delay)
thread_real_inmemory  thread_real_channels  thread_real_tcp  thread_real_grpc

# 2 process variants (Generous timer delay)
process_real_tcp      process_real_grpc
```

Each test creates a `ClusterBuilder`, selects spawn/timer/transport, and asserts
`cluster.poll_until_bootstrapped()` converges within the test timeout.

---

## Test directory structure

```
system-tests/tests/
├── all_tests.rs
├── convergence_tests.rs           ← 10-variant rstest
├── shared_file_observer_tests.rs  ← 9 tests
├── timer/
│   ├── mod.rs
│   └── real/
│       ├── mod.rs               ← pub mod real_timer_tests;
│       └── real_timer_tests.rs  ← 9 tests
└── transport/
    ├── mod.rs
    ├── channels/
    │   ├── mod.rs                ← pub mod channels_tests;
    │   └── channels_tests.rs    ← 6 tests
    ├── grpc/
    │   ├── mod.rs               ← pub mod grpc_tests;
    │   └── grpc_tests.rs        ← 7 tests
    ├── in_memory/
    │   ├── mod.rs               ← pub mod in_memory_tests;
    │   └── in_memory_tests.rs   ← 6 tests
    └── tcp/
        ├── mod.rs               ← pub mod tcp_tests;
        └── tcp_tests.rs         ← 7 tests
```

---

## Polling

For in-process nodes (Task/Thread), `Cluster` polls each node's state in a loop with
a backoff sleep matching the timer delay. For process nodes, `Cluster` blocks on
`child.wait()` — each process exits autonomously when bootstrapped.

---

## What this validates

| Layer | Validates | Covered by |
|---|---|---------|
| Machine correctness | Every `(state, command)` pair | Transition matrix (145 tests) |
| Invariants | Counts, exit, idempotency | Property tests (11 invariant + 4 safety) |
| Timer behavior | Schedule, poll, cancel, deadlines | Timer integration tests (9 tests) |
| Transport framing | Send/recv, FIFO, multi-peer | Transport integration tests (22 tests) |
| Observer logging | All event types, valid JSON | SharedFileObserver tests (9 tests) |
| Protocol convergence | 5-node cluster → bootstrapped | Convergence rstest (10 variants) |
| Process isolation | No shared-memory coupling | process_real_tcp, process_real_grpc |
