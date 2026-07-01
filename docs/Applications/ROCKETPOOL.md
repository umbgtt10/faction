# Rocket Pool GMC Grant Application — Round 38

---

## `faction` — Formally Specified Megapool Validator Lifecycle Management

### What is the work being proposed?

This grant funds Phases 1–4 of `faction`: a formally specified, completely
tested, protocol-agnostic Mealy state machine for distributed cluster
bootstrapping and validator lifecycle management.

The Saturn One upgrade introduced **megapools**: a single contract per node
acting as withdrawal credentials for multiple validators simultaneously. Every
Rocket Pool node operator running a megapool is now operating what is
structurally a small distributed cluster. The same fundamental question that
must be answered before any distributed cluster can operate now applies to
every megapool:

**"Is my validator cluster actually ready to start, and can I safely add or
remove a validator while it is running?"**

This is not answered by the Ethereum consensus clients. It is not answered by
the Rocket Pool smart contracts. It is answered — implicitly, without formal
specification, and without test coverage against partial-startup or
Byzantine-peer failure modes — by the Smartnode daemon's bespoke validator
lifecycle state machine.

`faction` provides the formally specified, completely tested reference
implementation for exactly this problem.

It implements a pure Mealy state machine: `output = F(state, input)`. No side
effects inside the machine. No network I/O. No opinion on the Smartnode's
internal architecture. The caller owns the network; `faction` owns the state
transitions. The Smartnode's existing Go lifecycle code can be validated
against `faction`'s transition matrix, or the matrix can serve as the formal
specification for a reimplementation.

**Phase 0** (complete, published on crates.io) delivers static membership
bootstrapping — the "is this cluster ready?" question — verified across 10
spawn/transport combinations including real OS processes over TCP and gRPC.

This grant funds **Phases 1 through 4**:

- **Phase 1 — Dynamic joining**: a validator can request admission to the
  megapool's active set at runtime. The machine signals the request; the
  Smartnode decides the admission policy; `faction` enforces it.

- **Phase 2 — Failure detection**: SWIM-style probing (suspect → indirect
  probe → confirm or revive), providing formally testable offline validator
  detection — not a timeout tuned by intuition.

- **Phase 3 — Single-node addition**: a commit/abort reconfiguration protocol
  for safe validator addition. Split-brain prevention by construction: it is
  structurally impossible for two disjoint subsets of the megapool's validators
  to each believe they form a valid quorum.

- **Phase 4 — Single-node removal**: symmetric to Phase 3, with harder
  invariants. The machine refuses removals that would drop the live set below
  quorum threshold. This is directly aligned with the forced-exit capability
  planned for Saturn 2: quorum preservation is guaranteed by the type system,
  not by convention.

The deliverable is not a paper describing correct behavior. It is the correct
behavior — formally specified, completely tested, and published on crates.io at
each phase boundary under the MIT license.

### Is there any related work this builds off of?

`faction` Phase 0 is the direct predecessor and is already complete and
published: https://crates.io/crates/faction

Phase 0 was extracted from **EtheRAM** — the applicant's Ethereum-compatible
distributed system running on real embedded hardware — when the bootstrapping
problem became clear enough to deserve its own formally specified solution.
The fact that `faction` is validated in a real distributed protocol, not an
abstract exercise, is what distinguishes it from a purely theoretical
contribution.

The research methodology — complete `(state × input)` coverage as a proof
strategy — is publicly verifiable in the transition matrix test file:
https://github.com/umbgtt10/faction/blob/main/core/tests/transition_matrix/state_transition_matrix_tests.rs

### Will the results of this project be entirely open source?

Yes. All phases of `faction` are and will remain published under the
**MIT license** on crates.io.

---

## Benefit

| Group | Benefits |
|---|---|
| Potential rETH holders | N/A directly. Indirectly: a more reliable validator lifecycle means fewer missed attestations, which supports rETH APY stability. |
| rETH holders | More reliable megapool validator startup and failure detection translates to fewer missed attestations and fewer penalties, which protects rETH APY. |
| Potential NOs | A formally correct validator lifecycle reference reduces the risk of cluster startup failures when setting up a new megapool — lowers the technical floor risk of running a multi-validator node for the first time. |
| NOs | Concrete reference for validating the Smartnode's megapool lifecycle logic against a formally tested state machine. Every megapool validator addition and exit has a formal correctness specification rather than implicit, untested Smartnode daemon behavior. Phase 4 is directly relevant to the forced-exit capability coming in Saturn 2. |
| Community | A formally correct, MIT-licensed primitive that the Smartnode team can adopt or use as a ground-truth specification. A bug found and fixed in `faction` is a class of bug that never surfaces for node operators. |
| RPL holders | More reliable node operation → fewer penalties → better protocol health → stronger TVL fundamentals. |

### Which other non-RPL protocols, DAOs, projects, or individuals, would stand to benefit from this grant?

- **Obol and SSV (DVT protocols)**: both operate validator clusters explicitly.
  `faction`'s cluster membership primitive is directly applicable.
- **Ethereum client teams** (Lighthouse, Prysm, Teku, Nimbus): the `(state ×
  input)` testing methodology is a transferable contribution any team building
  validator lifecycle state machines can adopt.
- Any distributed system project using Rust that needs a formally tested
  membership primitive can import `faction` today from crates.io.

---

## Work

### Who is doing the work?

Umberto Gotti — independent software engineer and contractor.

### What is the background of the person(s) doing the work?

15+ years of experience across embedded systems, medical technology, finance,
and defense. MSc in Computer Engineering, La Sapienza, Rome (110/110 —
highest distinction). ICP-ACC and iSAQB certifications.

Published crates directly relevant to this work and verifiable today:

- **`faction` (Phase 0)** — the subject of this application: 275 tests, 100%
  `(state, command)` coverage, 10-combination system test matrix (3 spawn
  models × 4 transport protocols: Memory, Channels, TCP, gRPC), CRAP score 0,
  zero unsafe code, `no_std` verified. Published MIT on crates.io.
- **`fluxion`** — Rust stream aggregation with formal ordering guarantees, MIT.
- **`crap4rust`** — CRAP score quality gate, used across all projects as a
  mandatory quality gate.

The execution track record is public and verifiable on crates.io and GitHub
without taking the applicant's word for it.

### What is the breakdown of the proposed work, in terms of milestones and/or deadlines?

Each phase is 4–6 weeks. No phase begins until the previous phase has 100%
`(state, command)` coverage and CRAP score 0. Phase 0 tests pass unchanged
at Phase 4.

| Phase | Deliverable | Duration | Rocket Pool relevance |
|---|---|---|---|
| Phase 1 — Dynamic joining | `faction` 0.4.x on crates.io | 4–6 weeks | A new validator can join an existing megapool cluster at runtime without restart |
| Phase 2 — Failure detection | `faction` 0.5.x on crates.io | 4–6 weeks | Formally testable offline validator detection — replaces implicit oDAO timeout logic with a provably correct state machine |
| Phase 3 — Single-node addition | `faction` 0.6.x on crates.io | 4–6 weeks | Safe validator addition to a running megapool with split-brain prevention by construction |
| Phase 4 — Single-node removal | `faction` 0.7.x on crates.io | 4–6 weeks | Safe validator exit with quorum preservation guaranteed by the type system — directly aligned with Saturn 2 forced exits |

Total duration: approximately 24–26 weeks from grant start.

### How is the work being tested? Is testing included in the schedule?

Testing is the primary deliverable, not a separate phase. Each release covers
every `(state, command)` pair in the complete transition matrix — not as a
sample but as a proof. This is not self-reported: the transition matrix test
file is public executable code that anyone can run.

Quality gates enforced at each phase boundary, all publicly verifiable:
- 100% `(state, command)` coverage — transition matrix test file
- multiple system/permutation tests — See [ARCHITECTURE.md](https://github.com/umbgtt10/faction/blob/main/docs/ARCHITECTURE.md)
- CRAP score 0 — `crap4rust` quality gate
- `#![deny(unsafe_code)]` — enforced at compile time, not by convention
- `no_std` — enforced at compile time, not by convention

### How will the work be maintained after delivery?

`faction` is published on crates.io with versioned releases at each phase
boundary. Post-delivery maintenance consists of responding to community issues
and bug reports on GitHub, ongoing as with all the applicant's published
crates. The MIT license means any team can also fork and maintain their own
copy. There is no infrastructure cost.

---

## Costs

### What is the acceptance criteria?

Each phase is accepted when:
1. The corresponding `faction` version is published on crates.io by the
   milestone deadline
2. The transition matrix test file covers every `(state, command)` pair for
   that phase (publicly verifiable)

No self-reporting required. The crates.io publish date and test count are the evidence.

### What is the proposed payment schedule for the grant? How much USD $ and over what period of time is the applicant requesting?

**Total requested: $100,000 USD, disbursed at time of each milestone payment.**

The payment structure is designed to give the GMC natural checkpoints and
off-ramps. No subsequent payment is made unless the preceding milestone's
quality gates pass:

| Milestone | Payment |
|---|---|
| Grant approval (Phase 1 begin) | $15,000 |
| Phase 1 complete — `faction` 0.4.x on crates.io | $20,000 |
| Phase 2 complete — `faction` 0.5.x on crates.io | $20,000 |
| Phase 3 complete — `faction` 0.6.x on crates.io | $22,500 |
| Phase 4 complete — `faction` 0.7.x on crates.io | $22,500 |

At Swiss contracting market rates for principal-level distributed systems
engineering, $100,000 represents approximately 6–7 months of sustainable
part-time work — less than 3 months of a senior engineer's fully-loaded cost
at any of the major Ethereum client teams, in exchange for a formally correct
primitive that addresses the megapool lifecycle problem for all of them
permanently.

**Alternative scope**: if the GMC prefers a smaller initial commitment, the
applicant is open to funding only Phases 1–2 ($55,000) as the primary grant,
with Phases 3–4 submitted as a follow-on application once Phase 2 delivery
demonstrates execution quality.

### Who will directly receive the payment?

Umberto Gotti — Ethereum wallet address: 0x6857fcdF0EE0bD6De569dD5dafB0374AE2901EaA

### How will the GMC verify that the work delivered matches the proposed cadence?

Every milestone deliverable is a publicly verifiable crates.io publication:
- **crates.io publish date** confirms the milestone was hit
- **Test count in the published crate** confirms coverage growth
- **Transition matrix test file** (linked from the crate README) is executable
  by anyone — run `cargo test` and every `(state, command)` pair either passes
  or fails

The GMC does not need to trust the applicant. The artifacts verify themselves.

### What alternatives or options have been considered in order to save costs for the proposed project?

The phased payment structure above means the GMC is not committed to $100k
upfront. Each milestone is independently verified before the next payment is
released. If execution quality falls below the stated standards at any phase,
subsequent payments are withheld — this is structurally equivalent to four
sequential smaller grants with automatic renewal conditioned on delivery.

The alternative scope (Phases 1–2 only, $55,000) is explicitly on the table
if the GMC prefers to validate Phase 1–2 delivery before committing to
Phases 3–4.

### Have you already been compensated by the RP protocol in any way for this work?

No.

---

## Conflict of Interest

### Does the person or persons proposing the grant have any conflicts of interest to disclose?

No GMC membership. No financial relationship with any GMC member.

One disclosure the GMC should be aware of: `faction` was originally developed
as part of **EtheRAM**, the applicant's personal Ethereum-compatible embedded
blockchain research project. EtheRAM uses `faction` as its cluster
bootstrapping primitive. This grant would accelerate `faction`'s development
in ways that also benefit EtheRAM. EtheRAM is a non-commercial research project
that generates no revenue. The applicant discloses this as a transparency
measure rather than a material conflict — the fact that `faction` is
battle-tested in a real distributed protocol is precisely what makes Phase 0's
quality metrics credible.

### Will the recipient of the grant, or any protocol or project in which the recipient has a vested interest (other than Rocket Pool), benefit financially if the grant is successful?

EtheRAM (the applicant's personal research project) would benefit from the
`faction` improvements funded by this grant. EtheRAM is not a commercial
entity, has no token, and generates no revenue. There is no financial benefit
to the applicant beyond the grant itself.

---

## Links

| Resource | URL |
|---|---|
| crates.io | https://crates.io/crates/faction |
| GitHub | https://github.com/umbgtt10/faction |
| ARCHITECTURE.md | https://github.com/umbgtt10/faction/blob/main/docs/ARCHITECTURE.md |
| ROADMAP.md | https://github.com/umbgtt10/faction/blob/main/docs/ROADMAP.md |
| Transition matrix (Phase 0) | https://github.com/umbgtt10/faction/blob/main/core/tests/transition_matrix/state_transition_matrix_tests.rs |
| `crap4rust` | https://crates.io/crates/crap4rust |
| `fluxion` | https://crates.io/crates/fluxion-rx |

---
