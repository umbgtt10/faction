# faction

**A `no_std`, 0-unsafe Mealy machine for two-phase cluster readiness coordination.**

faction is a protocol-agnostic state machine primitive that answers one question:  
*"When is the cluster ready to proceed?"*

It tracks participation and readiness signals across a known set of peers, applies configurable freshness classification, and emits a deterministic exit decision — either **Quorum** or **Deadline**. No network I/O. No consensus algorithm. No opinion on what "ready" means. Just pure, testable state transitions.

---

## Why faction?

Most distributed systems bootstrap with ad-hoc coordination — timeouts, magic numbers, and implicit assumptions that are never tested in isolation. faction replaces that with a **Mealy machine** that is:

- **Deterministic** — same inputs always produce the same outputs. Replay any sequence.
- **Verifiable** — every `(state, input)` pair is explicitly tested. No untested paths.
- **Embeddable** — `no_std` + `alloc`, zero `unsafe`. Runs on bare metal, WASM, and cloud.
- **Observable** — every transition reaches a trait-based `MachineObserver`. No instrumentation surprises.
- **Slim by construction** — each state owns only its active data. Frozen fields are inherited from the previous snapshot. Terminal states are lightweight counters.

---

## How it works

The machine progresses through four states:

```
Initial → Pinging → Collecting → ReadyByQuorum
                         ↘           ReadyByDeadline
```

| State | Carries |
|---|---|
| `Initial` | Nothing — unit struct |
| `Pinging` | Two active `ConfirmedSet`s (phase 1 + phase 2) |
| `Collecting` | One active `ConfirmedSet` (phase 2) + frozen phase 1 count |
| `ReadyByQuorum` | Two frozen `usize` counts |
| `ReadyByDeadline` | Two frozen `usize` counts |

Each state implements two traits:

- **`MachineState`** — transition logic (`step`) and input gating (`accept`)
- **`StateSnapshot`** — returns a delta over the previous snapshot. Fields the state doesn't touch are inherited automatically.

```rust
impl StateSnapshot for ReadyByDeadline {
    fn state_snapshot(&self, previous: &MachineSnapshot) -> MachineSnapshot {
        previous
            .with_lifecycle_state(ReadinessLifecycleState::ReadyByDeadline)
            .with_exit_mode(Some(ReadinessExitMode::Deadline))
            .with_readiness_exited(true)
            .with_phase1_count(self.phase1_count)
            .with_phase2_count(self.phase2_count)
            // local_participation_complete is inherited from `previous`
            // — true if entered from Collecting, false if from Pinging
    }
}
```

---

## Quick start

```rust
use faction::machine::Machine;
use faction::machine_config::MachineConfig;
use faction::machine_input::MachineInput;
use faction::freshness_policy::FreshnessPolicy;
use faction::quorum_policy::QuorumPolicy;
use faction::no_op_machine_observer::NoOpMachineObserver;

let mut machine = Machine::new(
    MachineConfig::new(
        0,                           // local peer ID
        vec![0, 1, 2, 3, 4],         // peer set
        QuorumPolicy::new(4),        // quorum threshold (of 5)
        FreshnessPolicy::new(2),     // max delay margin
    ),
    Box::new(NoOpMachineObserver),
);

// Feed observations — outputs are deterministic and replayable
let outputs = machine.apply(MachineInput::ParticipationObserved {
    peer_id: 1,
    freshness: 10,
    current_marker: 10,
});
```

---

## Roadmap

The project is building toward full dynamic membership across five phases.
See [ROADMAP.md](./docs/ROADMAP.md) for the detailed plan.

---

## Design principles

- **Pure Mealy** — `output = F(state, input)`. No side effects inside the machine.
- **Explicit state ownership** — states carry only what they mutate. Frozen data becomes a `usize` count, not a mutable collection.
- **No dead code** — terminal states return `false` from `accept()`, making `step()` unreachable by construction. No misleading match arms.
- **Observer, not logger** — the `MachineObserver` trait receives every transition. Wire it to telemetry, audit, or testing assertions.
- **Protocol-agnostic** — faction does not know what a "peer" is or how the network works. The caller owns network I/O.

---

## Workspace

| Crate | Description |
|---|---|
| `core/` | State machine — 17 source files, 164 tests, zero warnings |
| `validation/` | Deterministic multi-node scenario harness — 31 tests |

---

## Quality Gates

```powershell
powershell -File scripts\run_stage_1.ps1   # format, clippy, no_std checks, tests
powershell -File scripts\run_stage_2.ps1   # coverage and file risk analysis
```

---

## License

Licensed under the MIT License. See [LICENSE](./LICENSE).