## `faction` — Formally Specified Megapool Validator Lifecycle Management

### What is the work being proposed?

This grant funds Phases 1–4 of `faction`: a formally specified, completely
tested, protocol-agnostic Mealy state machine for distributed cluster
bootstrapping and node lifecycle management.

The Saturn One upgrade introduced megapools — a single contract per node acting
as withdrawal credentials for multiple validators simultaneously. Every Rocket
Pool node operator running a megapool is now operating what is structurally a
small distributed cluster. The same foundational question that must be answered
before any distributed cluster can operate now applies to every megapool:

**"Is my validator cluster actually ready to start, and can I safely add or
remove a validator while it is running?"**

This is not answered by the Ethereum consensus clients. It is not answered by
the Rocket Pool smart contracts. It is answered — implicitly, without formal
specification, and without test coverage against partial-startup or
Byzantine-peer failure modes — by the Smartnode daemon's bespoke validator
lifecycle state machine.

`faction` provides the formally specified, completely tested reference
implementation for exactly this problem. It is a pure Mealy state machine:
`output = F(state, input)`. No side effects. No network I/O. No opinion on the
Smartnode's internal architecture. The Smartnode's existing Go lifecycle code
can be validated against `faction`'s complete transition matrix, or the matrix
can serve as the formal specification for a reimplementation.

Phase 0 (complete, published MIT on crates.io) delivers static membership
bootstrapping — the "is this cluster ready?" question — verified across 10
spawn/transport combinations. This grant funds Phases 1–4:

- **Phase 1 — Dynamic joining**: a validator can request admission to the
  megapool's active set at runtime. The machine signals the request; the
  Smartnode decides the admission policy; `faction` enforces it.
- **Phase 2 — Failure detection**: SWIM-style probing (suspect → indirect probe
  → confirm or revive) with formally testable offline validator detection.
- **Phase 3 — Single-node addition**: commit/abort reconfiguration for safe
  validator addition with split-brain prevention by construction.
- **Phase 4 — Single-node removal**: safe validator exit with quorum
  preservation guaranteed by the type system — directly aligned with the
  forced-exit capability planned for Saturn 2.

### Is there any related work this builds off of?

`faction` Phase 0 is the direct predecessor and is complete and published:
https://crates.io/crates/faction

Phase 0 was extracted from **EtheRAM** — the applicant's Ethereum-compatible
distributed system running on real embedded hardware — when the bootstrapping
problem became clear enough to deserve its own formally specified solution. The
research methodology — complete `(state × input)` coverage as a proof strategy
— is publicly verifiable in the transition matrix test file:
https://github.com/umbgtt10/faction/blob/main/core/tests/transition_matrix/state_transition_matrix_tests.rs

### Will the results of this project be entirely open source?

Yes. All phases of `faction` are and will remain published under the **MIT
license** on crates.io.

---

## Benefit

| Group | Benefits |
|---|---|
| Potential rETH holders | N/A directly. Indirectly: more reliable megapool validator startup and failure detection means fewer missed attestations, which supports rETH APY stability. |
| rETH holders | More reliable megapool cluster operation translates to fewer missed attestations and fewer penalties, which protects rETH APY. |
| Potential NOs | A formally correct validator lifecycle reference reduces the risk of cluster startup failures when setting up a new megapool — lowers the technical floor risk of running a multi-validator node for the first time. |
| NOs | Concrete reference for validating the Smartnode's megapool lifecycle logic against a formally tested state machine. Every megapool validator addition and exit has a formal correctness specification rather than implicit, untested Smartnode daemon behavior. Phase 4 is directly relevant to the forced-exit capability coming in Saturn 2. |
| Community | A formally correct, MIT-licensed primitive that the Smartnode team can adopt or use as a ground-truth specification. A bug found and fixed in `faction` is a class of bug that never surfaces for node operators. |
| RPL holders | More reliable node operation → fewer penalties → better protocol health → stronger TVL fundamentals. |

### Which other non-RPL protocols, DAOs, projects, or individuals, would stand to benefit from this grant?

- **Obol and SSV (DVT protocols)**: both operate validator clusters explicitly.
  `faction`'s cluster membership primitive is directly applicable to DVT cluster
  bootstrapping and membership management.
- **Ethereum client teams** (Lighthouse, Prysm, Teku, Nimbus, Lodestar): the
  `(state × input)` testing methodology is a transferable contribution any team
  building validator lifecycle state machines can adopt.
- Any distributed systems project in Rust that needs a formally tested
  membership primitive can import `faction` from crates.io today.

---

## Work

### Who is doing the work?

Umberto Gotti — independent software engineer and contractor.

### What is the background of the person(s) doing the work?

15+ years of experience across embedded systems, medical technology, finance,
and defense. MSc in Computer Engineering, La Sapienza, Rome (110/110 — highest
distinction). Currently contracting at Hexagon (Leica Geosystems) as tech lead
on an embedded Jetson/Yocto project. ICP-ACC and iSAQB certified.

Published crates directly verifiable on crates.io and GitHub:

- **`faction` (Phase 0)**: 275 tests, 100% `(state, command)` coverage,
  10-combination system test matrix (3 spawn models × 4 transport protocols:
  Memory, Channels, TCP, gRPC), CRAP score 0, zero unsafe code, `no_std`
  verified. MIT.
- **`crap4rust`**: CRAP score quality gate, used as a mandatory CI gate across
  all projects. MIT.
- **`fluxion`**: Rust stream aggregation with formal ordering guarantees. MIT.

Beyond the tooling portfolio, the applicant is building **EtheRAM** — a minimal
distributed system (IBFT + Raft consensus, EVM execution) running on real
embedded hardware (Nucleo-F767ZI). `faction` was extracted from EtheRAM when
the bootstrapping problem became large enough to deserve its own formally
specified solution. EtheRAM is the validation environment that grounds
`faction`'s design decisions in a real distributed protocol.

I work independently. No team overhead, no coordination cost, no diffusion of
ownership. The person writing this application is the person who will implement
every line of code, write every test, and publish every crate version.

### What is the breakdown of the proposed work, in terms of milestones and/or deadlines?

Each phase is 4–6 weeks. No phase begins until the preceding phase has 100%
`(state, command)` coverage and CRAP score 0. Phase 0 tests pass unchanged at
Phase 4.

| Phase | Deliverable | Duration | Rocket Pool relevance |
|---|---|---|---|
| Phase 1 — Dynamic joining | `faction` 0.4.x on crates.io | 4–6 weeks | A new validator can join an existing megapool cluster at runtime without restart |
| Phase 2 — Failure detection | `faction` 0.5.x on crates.io | 4–6 weeks | Formally testable offline validator detection — replaces implicit Smartnode timeout logic with a provably correct state machine |
| Phase 3 — Single-node addition | `faction` 0.6.x on crates.io | 4–6 weeks | Safe validator addition to a running megapool with split-brain prevention by construction |
| Phase 4 — Single-node removal | `faction` 0.7.x on crates.io | 4–6 weeks | Safe validator exit with quorum preservation guaranteed by the type system — aligned with Saturn 2 forced exits |

Total duration: approximately 24–26 weeks from grant start.

### How is the work being tested? Is testing included in the schedule?

Testing is the primary deliverable at every phase, not a separate step. Each
release covers every `(state, command)` pair in the complete transition matrix —
not as a sample but as a proof. This is not self-reported: the transition matrix
test file is public, executable code that anyone can run with `cargo test`.

Quality gates enforced at every phase boundary, all publicly verifiable:
- 100% `(state, command)` coverage — transition matrix test file
- CRAP score 0 — `crap4rust` quality gate
- `#![forbid(unsafe_code)]` — enforced at compile time
- `no_std` — verified by CI check script
- permutation system tests (tasks, threads and processes) — 10 spawn/transport combinations

### How will the work be maintained after delivery?

`faction` is published on crates.io with versioned releases at each phase
boundary. Post-delivery maintenance consists of responding to community issues
and bug reports on GitHub — ongoing, as with all the applicant's published
crates. The MIT license means any team can also fork and maintain their own
copy independently.

---

## Costs

### What is the acceptance criteria?

Each phase is accepted when:
1. The corresponding `faction` version is published on crates.io
2. The transition matrix test file covers every `(state, command)` pair for
   that phase (publicly verifiable — run `cargo test`)
3. `crap4rust` reports CRAP score 0
4. The `no_std` and #![forbid(unsafe_code)]` enforced by the compiler
5. The 10+ spawn/transport system tests pass

No self-reporting required. The crates.io publish date, test count, and CI
results are the evidence.

### What is the proposed payment schedule for the grant? How much USD $ and over what period of time is the applicant requesting?

**Total requested: $100,000 USD, disbursed in RPL at time of each milestone
payment.**

| Milestone | Payment |
|---|---|
| Grant approval | $15,000 |
| Phase 1 complete — `faction` 0.4.x on crates.io | $20,000 |
| Phase 2 complete — `faction` 0.5.x on crates.io | $20,000 |
| Phase 3 complete — `faction` 0.6.x on crates.io | $22,500 |
| Phase 4 complete — `faction` 0.7.x on crates.io | $22,500 |

No subsequent payment is released until the preceding milestone's quality gates
pass. Total duration approximately 24–26 weeks.

If the GMC prefers a smaller initial commitment, Phases 1–2 ($55,000) are a
natural standalone scope — dynamic joining and failure detection — with Phases
3–4 submitted as a follow-on application after Phase 2 delivery demonstrates
execution quality.

### Who will directly receive the payment?

Umberto Gotti — Ethereum wallet address: 0x6857fcdF0EE0bD6De569dD5dafB0374AE2901EaA

### How will the GMC verify that the work delivered matches the proposed cadence?

Every milestone deliverable is a publicly verifiable crates.io publication:
- **crates.io publish date** confirms the milestone was hit on schedule
- **Transition matrix test file** (linked from the crate README) is executable
  by anyone — `cargo test` either passes every `(state, command)` pair or does
  not
- **`crap4rust` ** runnable from the command line
- **`#![forbid(unsafe_code)]` and `no_std` ** enforced by the compiler

The GMC does not need to trust the applicant. The artifacts verify themselves.

### What alternatives or options have been considered in order to save costs for the proposed project?

The milestone-gated payment structure means the GMC is not committed to $100k
upfront. Each payment is independently conditioned on verifiable delivery. If
execution quality falls below the stated standards at any phase, subsequent
payments are withheld.

The Phases 1–2 only scope ($55,000) is explicitly on the table if the GMC
prefers to validate Phase 1–2 delivery before committing to Phases 3–4.

### Have you already been compensated by the RP protocol in any way for this work?

No.

---

## Conflict of Interest

### Does the person or persons proposing the grant have any conflicts of interest to disclose?

No GMC membership. No financial relationship with any GMC member.

One disclosure: `faction` was originally developed as part of **EtheRAM**, the
applicant's personal Ethereum-compatible embedded blockchain research project.
EtheRAM uses `faction` as its cluster bootstrapping primitive. This grant
accelerates `faction`'s development in ways that also benefit EtheRAM. EtheRAM
is a non-commercial research project that generates no revenue. The applicant
discloses this as a transparency measure — the EtheRAM validation environment
is precisely what makes Phase 0's quality metrics credible.

### Will the recipient of the grant, or any protocol or project in which the recipient has a vested interest (other than Rocket Pool), benefit financially if the grant is successful?

EtheRAM (the applicant's personal research project) would benefit from the
`faction` improvements funded by this grant. EtheRAM is not a commercial entity,
has no token, and generates no revenue. There is no financial benefit to the
applicant beyond the grant itself.

---

## Links

| Resource | URL |
|---|---|
| crates.io | https://crates.io/crates/faction |
| GitHub | https://github.com/umbgtt10/faction |
| Full application | https://github.com/umbgtt10/faction/blob/main/docs/Applications/ROCKETPOOL.md |
| Transition matrix | https://github.com/umbgtt10/faction/blob/main/core/tests/transition_matrix/state_transition_matrix_tests.rs |
| `crap4rust` | https://crates.io/crates/crap4rust |
| `fluxion` | https://crates.io/crates/fluxion-rx |
