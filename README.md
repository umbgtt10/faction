# faction

**A `no_std`, 0-unsafe Mealy machine for two-phase cluster readiness coordination.**

faction is a protocol-agnostic state machine primitive that answers one question:  
*"When is the cluster ready to proceed?"*

It tracks participation and readiness signals across a known set of peers, applies configurable freshness classification, and emits a deterministic exit decision — either **Bootstrapped** or **TimedOut**. No network I/O. No consensus algorithm. No opinion on what "ready" means. Just pure, testable state transitions.

---

## Why faction?

Most distributed systems bootstrap with ad-hoc coordination — timeouts, magic numbers, and implicit assumptions that are never tested in isolation. faction replaces that with a **Mealy machine** that is:

- **Deterministic** — same inputs always produce the same outputs. Replay any sequence.
- **Verifiable** — every `(state, input)` pair is explicitly tested. No untested paths.
- **Embeddable** — `no_std` + `alloc`, zero `unsafe`. Runs on bare metal, WASM, and cloud.
- **Observable** — every transition reaches a trait-based `Observer`. No instrumentation surprises.
- **Slim by construction** — each state carries only its active data. Terminal states are 8 bytes each.

---

## How it works

The machine progresses through five states:

```
Initial → Pinging → Collecting → Bootstrapped
                         ↘           TimedOut
```

| State | Carries |
|---|---|
| `Initial` | Nothing — unit struct |
| `Pinging` | `pinged_peers: Vec<PeerId>`, `collected_peers: Vec<PeerId>` |
| `Collecting` | `collected_peers: Vec<PeerId>`, `pinged_peers_count: usize` |
| `Bootstrapped` | `pinged_peers_count: usize`, `collected_peers_count: usize` |
| `TimedOut` | `pinged_peers_count: usize`, `collected_peers_count: usize` |

Each state implements the `State` trait with `step()`, `cluster_view()`, and `accept()`.
Decision logic is centralized in **`ObservedStep`** — a struct that takes a freshness
classification, the current confirmed peers, a peer identity, an observation kind, and a
quorum threshold, and returns the updated peer list plus outputs.

Five commands drive the machine: `ParticipationObserved`, `ReadyObserved`, `LocalParticipationCompleted`,
`DeadlineExpired`, and `Probe`. Thirteen outcomes cover acceptance, delay, staleness, duplication,
non-member rejection, local participation completion, broadcast, quorum, and exit.

Full specification in [phase-0-specification.md](./docs/phase-0-specification.md).

---

## Project status

| Metric | Value |
|---|---|
| Productive LOC (core) | 1,312 |
| Tests (core + validation) | 216 |
| Crappy functions | 0 / 114 |
| Code coverage | 99.7% (100% effective) |
| Unsafe | 0 |
| `no_std` | Verified |

---

## Roadmap

The project is building toward full dynamic membership across six phases.
See [ROADMAP.md](./docs/ROADMAP.md) for the detailed plan.

---

## Workspace

| Crate | Description |
|---|---|
| `core/` | State machine — 13 source files, 192 tests |
| `validation/` | Deterministic multi-node scenario harness — 34 tests |

---

## Quality Gates

```powershell
powershell -File scripts\run_stage_1.ps1   # format, clippy, no_std checks, tests
powershell -File scripts\run_stage_2.ps1   # CRAP and file risk analysis
```

---

## Design principles

- **Pure Mealy** — `output = F(state, input)`. No side effects inside the machine.
- **Explicit state ownership** — states carry only what they mutate. Counts become `usize`, not mutable collections.
- **No dead code** — terminal states return `false` from `accept()`, making `step()` unreachable by construction.
- **Observer, not logger** — the `Observer` trait receives every transition. Wire it to telemetry, audit, or testing assertions.
- **Protocol-agnostic** — faction does not know what a "peer" is or how the network works. The caller owns network I/O.

---

## License

Licensed under the MIT License. See [LICENSE](./LICENSE).

---

## Links

- [CHANGELOG](./CHANGELOG.md) — project history
- [CODE_OF_CONDUCT](./CODE_OF_CONDUCT.md) — community guidelines
- [DONATE](./DONATE.md) — support the project
- [ROADMAP](./docs/ROADMAP.md) — future plans
- [Phase 0 specification](./docs/phase-0-specification.md) — detailed state machine description
