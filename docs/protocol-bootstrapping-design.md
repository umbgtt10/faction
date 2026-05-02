# Protocol bootstrapping — Design

**Status:** Implemented  
**Crate:** `faction-protocol`  
**Depends on:** `faction` (core)

---

## Goal

Define the message exchange and timer strategy required for N nodes to reach
the `Bootstrapped` state autonomously, with no external orchestrator seeding
messages after startup.

---

## Normal flow

```
Node A                          Node B
  │                               │
  │ start_decisions()             │ start_decisions()
  ├── Schedule(Participation)     ├── Schedule(Participation)
  ├── Schedule(LocalComplete)     ├── Schedule(LocalComplete)
  │                               │
  │ Timer fires Participation     │ Timer fires Participation
  │ Timer fires LocalComplete     │ Timer fires LocalComplete
  ├── decide → BroadcastReady     │
  ├── Schedule(RetryReady)        │
  │ ── Ready(A) ─────────────────►│
  │                               ├── decide → BroadcastReady
  │                               ├── Schedule(RetryReady)
  │                               │ ── Ready(B) ─────────────────►
  │                               │                               │
  │ ◄── Ready(B) ─────────────────┤                               │
  ├── decide (quorum?)            │                               │
  │                               │ ◄── Ready(A) ─────────────────┤
  │                               ├── decide (quorum?)            │
  │                               │                               │
  ▼ Bootstrapped                  ▼ Bootstrapped
```

---

## Architecture

`Protocol` owns a `Faction` state machine and a `MessageTranslator`.
It does **not** track previous state or detect transitions — it delegates
outcome→output mapping to `MessageTranslator`.

```rust
pub struct Protocol {
    faction: Faction,
    peers: Vec<PeerId>,
    local_peer_id: PeerId,
    translator: MessageTranslator,
}
```

### `MessageTranslator`

A pure, stateless translator with two methods:

- **`to_command(InputMessage) -> Command`** — maps every transport/timer message
  variant to its corresponding Faction command (7 arms, exhaustively tested).

- **`to_output_messages(Vec<Outcome>) -> Vec<OutputMessage>`** — scans the outcome
  vector and returns on the first meaningful match:
  | Outcome | Output |
  |---|---|
  | `BroadcastLocalReady` | `[BroadcastReady, Schedule(RetryReady)]` |
  | `Concluded { .. }` | `[Cancel(LocalParticipationCompleted), Cancel(RetryReady)]` |
  | Everything else | `[Noop]` |

### `Protocol::decide()`

```
decide(input):
    if input is RetryReady:
        if exited → [Noop]
        else      → [BroadcastReady, Schedule(RetryReady)]

    command  = translator.to_command(input)
    outcomes = faction.process(command)
    if rejected → [Noop]
    return translator.to_output_messages(outcomes)
```

`RetryReady` is intercepted before the Faction sees it — the Faction never
receives a `RetryReady` command (the `to_command` mapping panics with
`unreachable!` if it ever does).

### `Protocol::initialize()`

Schedules exactly:
- One `ParticipationObserved` timer per remote peer
- One `LocalParticipationCompleted` timer

No `DeadlineExpired` is scheduled here — the deadline is injected externally
by the node runtime when it decides a global timeout is needed.

---

## Scenarios

### S1 — Normal convergence ✅

All nodes start, timers fire, nodes exchange Ready signals, reach quorum.

**Covered by:** `vanilla_convergence_tests` (5 nodes, quorum 4) and
`five_nodes_converge_to_bootstrapped` system test.

### S2 — Lost Ping ✅

`RetryPing` timer scheduled on `initialize()`; each fire produces
`[BroadcastPing, Schedule(RetryPing)]`. If a `Ping` transport message
is lost, the periodic retry ensures the peer's participation eventually
reaches all nodes.

**Covered by:** `decide_retry_ping_while_active_produces_broadcast_and_retry`
protocol test.

### S3 — Lost Ready ✅

`RetryReady` timer re-broadcasts `Ready` periodically until exit.

**Covered by:** `dropped_ready_tests` (2 nodes, quorum 2, one Ready dropped)
and `decide_retry_ready_while_active_produces_broadcast_and_retry` protocol test.

### S4 — Late arrival ✅

A `Ready` transport message can arrive before `LocalParticipationCompleted`.
The Faction's `Pinging` state accepts `ReadyObserved` and accumulates it in
`collected_peers`. When `LocalParticipationCompleted` later fires, those
pre-collected Ready signals count toward quorum.

**Covered by:** Protocol test `decide_ready_before_local_completion_is_noop`
(Ready accepted but produces Noop output in Pinging state).

### S5 — Deadline expired ✅

The node runtime injects `DeadlineExpired` as a timer message. The Protocol
maps it to `Command::DeadlineExpired`, the Faction transitions to `TimedOut`,
and `to_output_messages` returns `[Cancel(LPC), Cancel(RetryReady)]`.

**Covered by:** Protocol test `decide_deadline_expired_exits`.

### S6 — Duplicate suppression ✅

Handled entirely by the Faction core state machine. Duplicate
signals produce `*Ignored` outcomes, which fall through in
`to_output_messages` → `[Noop]`. No spurious re-broadcasts.

### S7 — Concluded exit cancellation ✅

When `Concluded` outcome is produced (Bootstrapped or TimedOut),
`to_output_messages` returns `[Cancel(LPC), Cancel(RetryReady)]`.
`RetryReady` also short-circuits via `is_exited()` check in `decide()`.

**Covered by:** `decide_retry_ready_while_exited_produces_noop` protocol test.

---

## Messages

### Transport messages (wire protocol)

| Message | Meaning | Produced by |
|---|---|---|
| `Ping { from }` | "I exist, here's my participation signal" | Node runtime / timer |
| `Ready { from }` | "I completed local participation" | `BroadcastReady` → dispatcher sends to all peers |
| `Bootstrapped { from }` | "I reached quorum" | Future (maps to `Probe` — currently panics in `decide()`) |

### Timer messages (scheduled events)

| Message | Meaning | Scheduled on | Cancelled on |
|---|---|---|---|
| `ParticipationObserved { peer_id }` | Seed a participation signal for a peer | `start_decisions()` | Never |
| `LocalParticipationCompleted` | Fire local completion | `start_decisions()` | Exit (`Exited` → Cancel) |
| `RetryReady` | Resend Ready to all peers | `BroadcastLocalReady` outcome | Exit (`Exited` → Cancel) |
| `DeadlineExpired` | Global timeout fired | Injected externally by node runtime | Exit (`Exited` → Cancel) |

### Output messages (Protocol → Dispatcher)

| Output | Meaning | Dispatcher action |
|---|---|---|
| `BroadcastReady` | Tell all peers I'm ready | Send `Ready { from: local_peer_id }` to every other peer |
| `Schedule(TimerEvent)` | Arm a timer | `timer.schedule(event)` |
| `Cancel(TimerEvent)` | Disarm a timer | `timer.cancel(event)` |
| `Noop` | Nothing to do | — |

---

## State transitions that produce outputs

### Any → Pinging

Triggered by: `ParticipationObserved` or `ReadyObserved` from a peer.

Produces: `[Noop]` — the Faction transitions internally but no protocol-level
output is emitted. The dispatcher is responsible for broadcasting pings on
first activation.

### Pinging → Collecting

Triggered by: `LocalParticipationCompleted`.

Produces:
- `BroadcastReady` — tell peers this node is ready
- `Schedule(RetryReady)` — periodic retry until exit

### Collecting → Bootstrapped

Triggered by: `ReadyObserved` from enough peers to meet quorum.

Produces:
- `Cancel(LocalParticipationCompleted)` — no longer needed
- `Cancel(RetryReady)` — quorum reached, stop retrying
- `Cancel(RetryPing)` — no longer needed

### Any → TimedOut

Triggered by: `DeadlineExpired`.

Produces:
- `Cancel(LocalParticipationCompleted)`
- `Cancel(RetryReady)`

---

## Retry strategy

| Timer | Mechanism | Termination |
|---|---|---|
| `RetryReady` | `BroadcastLocalReady` outcome schedules it; each fire produces `[BroadcastReady, Schedule(RetryReady)]` | Cancelled on exit (`Exited` outcome) or short-circuited by `is_exited()` check |

Retry interval and backoff are the responsibility of the `Timer` implementation,
not the Protocol.
