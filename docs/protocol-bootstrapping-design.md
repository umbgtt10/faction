# Protocol bootstrapping — Design

**Status:** Draft  
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
  │ ◄── Ping(B) ──────────────────┤  Timer fires LocalComplete
  ├── decide → BroadcastReady     │
  │ ── Ready(A) ─────────────────►│
  │                               ├── decide → BroadcastReady
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

## Scenarios

### S1 — Normal convergence

All nodes start, exchange Pings and Ready signals, reach quorum.

**Expected:** Both nodes reach `Bootstrapped` after exchanging `Ping` + `Ready`.

### S2 — Lost Ping

Node A sends Ping to Node B. The message is lost.

**Expected:** RetryPing timer fires on Node A, Ping is resent. Node B receives the retry.
Node B eventually receives it and the flow proceeds normally.

### S3 — Lost Ready

Node A sends Ready to Node B. The message is lost.

**Expected:** RetryReady timer fires on Node A, Ready is resent. Node B receives the retry.
Quorum is reached eventually.

### S4 — Late arrival

Node A starts, completes participation, broadcasts Ready. Node B starts later.

**Expected:** Node B receives Ping from its own timer, then receives Ready from Node A
(who already sent it). Node B processes Node A's Ready and responds with its own Ready.
Both converge.

### S5 — Deadline expired

No quorum forms within the global deadline.

**Expected:** DeadlineExpired fires on each node. Each exits with `TimedOut`.
All retry timers are cancelled.

### S6 — Stale/duplicate suppression

A retry timer fires after the signal was already received and processed.

**Expected:** The machine emits `Duplicate*Ignored` and does not change state.
No additional broadcasts are triggered by the retry.

### S7 — Premature exit cancellation

A node reaches Bootstrapped or TimedOut.

**Expected:** All pending timers (RetryPing, RetryReady, DeadlineExpired) are cancelled.
No further messages are produced.

---

## Messages

### Transport messages (wire protocol)

| Message | Meaning | Produced by |
|---|---|---|
| `Ping { from }` | "I exist, here's my participation signal" | `BroadcastPing` → dispatcher sends to all peers |
| `Ready { from }` | "I completed local participation" | `BroadcastReady` → dispatcher sends to all peers |
| `Bootstrapped { from }` | "I reached quorum" | Future: Phase 3+ |

### Timer messages (scheduled events)

| Message | Meaning | Scheduled on | Cancelled on |
|---|---|---|---|
| `ParticipationObserved { peer_id }` | Seed a participation signal for a peer | `start_decisions()` | Never |
| `LocalParticipationCompleted` | Fire local completion | `start_decisions()` | Never |
| `RetryPing` | Resend Ping to all peers | Initial → Pinging transition | Exit or quorum |
| `RetryReady` | Resend Ready to all peers | Pinging → Collecting transition | Exit or quorum |
| `DeadlineExpired` | Global timeout fired | `start_decisions()` | Quorum |

### Output messages (Protocol → Dispatcher)

| Output | Meaning | Dispatcher action |
|---|---|---|
| `BroadcastPing` | Tell all peers I exist | Send `Ping { from: local_peer_id }` to every other peer |
| `BroadcastReady` | Tell all peers I'm ready | Send `Ready { from: local_peer_id }` to every other peer |
| `Schedule(TimerEvent)` | Arm a timer | `timer.schedule(event)` |
| `Cancel(TimerEvent)` | Disarm a timer | `timer.cancel(event)` |
| `Noop` | Nothing to do | — |

---

## State transitions that produce outputs

### Initial → Pinging

Triggered by: `ParticipationObserved` or `ReadyObserved` from a peer.

Produces:
- `BroadcastPing` — tell peers this node just activated
- `Schedule(TimerEvent::Fire(RetryPing))` — retry pings
- `Schedule(TimerEvent::Fire(DeadlineExpired))` — global deadline (if not already scheduled)

### Pinging → Collecting

Triggered by: `ParticipationObserved` from enough peers + `LocalParticipationCompleted`.

Produces:
- `BroadcastReady` — tell peers this node is ready
- `Schedule(TimerEvent::Fire(RetryReady))` — retry ready broadcasts
- `Cancel(TimerEvent::Fire(RetryPing))` — no need to retry pings anymore

### Collecting → Bootstrapped

Triggered by: `ReadyObserved` from enough peers to meet quorum.

Produces:
- `Cancel(TimerEvent::Fire(RetryReady))` — quorum reached, stop retrying
- `Cancel(TimerEvent::Fire(DeadlineExpired))` — quorum reached, no deadline needed
- `BroadcastBootstrapped` — tell peers (Phase 3+ only)

### Any → TimedOut

Triggered by: `DeadlineExpired`.

Produces:
- `Cancel(TimerEvent::Fire(RetryPing))`
- `Cancel(TimerEvent::Fire(RetryReady))`
- `Cancel(TimerEvent::Fire(DeadlineExpired))`

---

## Implication for Protocol::decide()

`decide()` must return `Vec<OutputMessage>` instead of a single `OutputMessage`.
A single input can trigger multiple outputs (e.g., state transition + timer scheduling).

The Protocol needs to track the previous state internally to detect transitions:

```rust
pub struct Protocol {
    faction: Faction,
    peers: Vec<PeerId>,
    local_peer_id: PeerId,
    previous_state: PeerState,
}

impl Protocol {
    pub fn decide(&mut self, message: InputMessage) -> Vec<OutputMessage> {
        let previous = self.previous_state;
        let outcomes = ...; // process the message
        let current = self.faction.cluster_view().peer_state();
        self.previous_state = current;

        let mut outputs = Vec::new();

        // Core decision: what does this specific message produce?
        match core_outcome {
            BroadcastLocalReady => outputs.push(OutputMessage::BroadcastReady),
            Exited { .. } => outputs.push(OutputMessage::Cancel(...)),
            _ => {}
        }

        // Transition detection: what changed between previous and current?
        if previous == PeerState::Fresh && current == PeerState::Pinging {
            outputs.push(OutputMessage::BroadcastPing);
            outputs.push(OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryPing)));
            outputs.push(OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::DeadlineExpired)));
        }
        if previous == PeerState::Pinging && current == PeerState::Collecting {
            outputs.push(OutputMessage::BroadcastReady);
            outputs.push(OutputMessage::Schedule(TimerEvent::Fire(TimerMessage::RetryReady)));
            outputs.push(OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryPing)));
        }
        if current == PeerState::Bootstrapped {
            outputs.push(OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryReady)));
            outputs.push(OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::DeadlineExpired)));
        }
        if current == PeerState::TimedOut {
            outputs.push(OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryPing)));
            outputs.push(OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::RetryReady)));
            outputs.push(OutputMessage::Cancel(TimerEvent::Fire(TimerMessage::DeadlineExpired)));
        }

        outputs
    }
}
```

---

## Retry strategy

| Timer | Interval | Max retries | Backoff |
|---|---|---|---|
| `RetryPing` | 1s | 3 | 2× |
| `RetryReady` | 1s | 5 | 2× |
| `DeadlineExpired` | 30s | 1 | N/A |

Retry count tracking is the responsibility of the `Timer` implementation.
The Protocol only decides *what* to schedule and *when* to cancel.
