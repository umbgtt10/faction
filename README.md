# faction

**A `no_std`, 0-unsafe Mealy machine for cluster bootstrapping.**

faction is a protocol-agnostic state machine primitive that answers one question:  
*"When is the cluster ready to proceed?"*

It tracks participation and readiness signals across a known set of peers and emits a
deterministic exit decision — either **Bootstrapped** or **TimedOut**. No network I/O.
No consensus algorithm. No opinion on what "ready" means. Just pure, testable state
transitions.

---

## Why faction?

Most distributed systems bootstrap with ad-hoc coordination — timeouts, magic numbers,
and implicit assumptions that are never tested in isolation. faction replaces that with a
**Mealy machine** that is:

- **Deterministic** — same inputs always produce the same outputs. Replay any sequence.
- **Verifiable** — every `(state, command)` pair is explicitly tested. No untested paths.
- **Embeddable** — `no_std` + `alloc`, zero `unsafe`. Runs on bare metal, WASM, and cloud.
- **Observable** — every transition reaches a trait-based `Observer`. No instrumentation surprises.
- **Slim by construction** — each state carries only its active data. Terminal states carry counts, not collections.

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
| `Pinging` | `pinging_peers: Vec<PeerId>`, `collecting_peers: Vec<PeerId>` |
| `Collecting` | `collecting_peers: Vec<PeerId>`, `pinged_peers: Vec<PeerId>` |
| `Bootstrapped` | `pinged_peers: Vec<PeerId>`, `collected_peers: Vec<PeerId>` |
| `TimedOut` | `pinging_peers: Vec<PeerId>`, `collecting_peers: Vec<PeerId>` |

Each state implements the `State` trait with `step()`, `cluster_view()`, and `accept()`.
Decision logic is split into focused step structs — `PingingStep`, `ReadyStep`, and
`LocalCompletionStep` — each handling exactly one kind of observation without branching
on a kind enum.

Five commands drive the machine: `ParticipationObserved`, `ReadyObserved`,
`LocalParticipationCompleted`, `DeadlineExpired`, and `Probe`. Outcomes cover acceptance,
duplication, non-member rejection, local participation completion, broadcast, quorum,
and exit.

Full specification in [phase-0-specification.md](./docs/phase-0-specification.md).

---

## Project status

| Metric | Value |
|---|---|
| Productive LOC (core) | 1,165 |
| Tests (entire codebase) | 275 |
| Crappy functions | 0 |
| Code coverage (productive) | 100% |
| Unsafe | 0 |
| `no_std` | Verified |
| Drop impls (transports) | 4/4 |

---

## Roadmap

The project is building toward full dynamic membership across six phases.
See [ROADMAP.md](./docs/ROADMAP.md) for the detailed plan.

---

## Workspace

| Crate | Description |
|---|---|
| `core/` | State machine — 13 source files, 145 tests |
| `core-validation/` | Deterministic multi-node scenario harness — 23 tests |
| `protocol/` | Message translator and protocol runtime — 33 tests |
| `protocol-validation/` | In-process protocol cluster — 9 tests |
| `system-tests/` | Multi-process convergence + timer + transport + observer tests — 65 tests |

---

## Quality Gates

```powershell
powershell -File scripts\run_stage_1.ps1   # format, clippy, no_std checks, tests
powershell -File scripts\run_stage_2.ps1   # CRAP and file risk analysis
```

---

## Design principles

- **Pure Mealy** — `output = F(state, input)`. No side effects inside the machine.
- **Explicit state ownership** — states carry only what they mutate.
- **No dead code** — terminal states return `false` from `accept()`, making `step()` unreachable by construction.
- **Observer, not logger** — the `Observer` trait receives every transition. Wire it to telemetry, audit, or testing assertions.
- **Protocol-agnostic** — faction does not know what a "peer" is or how the network works. The caller owns network I/O.
- **One struct per file** — each step, state, and policy is its own file.
- **No `&mut` parameters** — prefer return values over in-place mutation.

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
- [Protocol bootstrapping design](./docs/protocol-bootstrapping-design.md) — protocol layer design
- [System tests design](./docs/system-tests-design.md) — multi-process test infrastructure
