## LEGO Grant Proposal — `faction`: Formally Specified DVT Cluster Bootstrapping and Validator Lifecycle Management

**Applicant:** Umberto Gotti
**Amount requested:** $100,000 USDC
**Grant tier:** Boulder
**Date:** July 2026

---

### Proposal Summary

`faction` is a formally specified, completely tested, protocol-agnostic Mealy
state machine for distributed cluster bootstrapping and validator lifecycle
management. I am requesting $100,000 USDC from LEGO to fund Phases 1–4 of the
`faction` roadmap.

Phase 0 — static cluster bootstrapping — is already complete, published on
crates.io under the MIT license, and verified across 10 spawn/transport
combinations. This proposal funds the four phases that deliver the complete DVT
cluster membership lifecycle: dynamic operator joining, failure detection, safe
operator addition, and safe operator removal. These are the four membership
events that Lido's 22,233 DVT validators — representing 711,456 ETH — currently
handle with bespoke, informally specified, untested code inside Obol's Charon
middleware and the SSV operator client.

---

### The Problem

**Lido has placed 711,456 ETH on DVT reliability. The cluster bootstrapping
layer has no formal specification.**

DVT cluster operation has three distinct phases:

**Phase A — Distributed Key Generation**: the cryptographic setup. Handled by
the Obol Launchpad / DKG ceremony and its SSV equivalent. This phase is well
understood and well documented.

**Phase B — Cluster bootstrapping**: the operational readiness question. After
DKG, and at every subsequent restart, the cluster must answer: are all nodes
running, mutually connected, and ready to begin signing duties? This is
structurally separate from DKG. The shared key exists. The question is whether
the cluster that will use it is actually ready. This phase is handled
implicitly — by bespoke startup logic inside Charon and the SSV client that is
not formally specified and is not tested against the class of failures it is
meant to survive.

**Phase C — Ongoing membership management**: failure detection, operator
replacement, and safe cluster reconfiguration while signing duties continue.
This phase is also handled by bespoke code, with no formal specification and no
test suite for the membership state machine.

Phases B and C are the gap. When Charon or the SSV client gets Phase B wrong,
the cluster fails to form. The failure is silent and indistinguishable from a
network partition or a misconfigured peer. There is no specification to debug
against. When Phase C is handled incorrectly — whether in a timeout that fires
too early, a missing quorum check, or a race between two concurrent membership
events — the consequences can include missed attestations, incorrect liveness
assessments, or in the worst case, split-brain conditions where two disjoint
subsets of the cluster each believe they hold quorum.

The LNOSG tracks exactly these symptoms when evaluating cluster performance:
uptime, validator duty completion, response times, ability to self-diagnose and
troubleshoot. The underlying cause of poor performance in these metrics is
frequently a bootstrapping or membership management failure in Phase B or C —
not a fault in the consensus client, not a network issue, but a gap in the
cluster's operational state machine.

As of Q4 2025, DVT powers 4x more validators than a year prior. The Curated
Module has now adopted DVT. The CSM has seen 35% voluntary DVT adoption among
reporting operators. The scale of this infrastructure — and the rate at which it
is growing — has significantly outpaced the formalization of the cluster
lifecycle it depends on. That gap is what this proposal addresses.

---

### The Solution — `faction`

`faction` is the formally specified, completely tested reference implementation
for DVT cluster bootstrapping and membership management. It fills Phases B and C
directly.

It implements a pure Mealy state machine: `output = F(state, input)`. No side
effects inside the machine. No network I/O. No opinion on Obol's or SSV's
internal architecture. The caller owns the network; `faction` owns the state
transitions.

**How Obol and SSV teams use it**

Both Charon and the SSV operator client can validate their existing cluster
lifecycle logic against `faction`'s complete transition matrix. The transition
matrix is not a document — it is executable code:
[state_transition_matrix_tests.rs](https://github.com/umbgtt10/faction/blob/main/core/tests/transition_matrix/state_transition_matrix_tests.rs).

Run `cargo test` against this file and every `(state, command)` pair either
passes or fails. Each failure is a gap between the reference implementation and
the expected behavior. Each gap is a potential cluster incident that is now
findable before production, not after. This is a directly actionable quality
check that any DVT engineer can run today against Phase 0's bootstrapping
transitions, and will be able to run against each subsequent phase as it ships.

**The testing methodology as the core contribution**

`faction` introduces a testing approach that does not currently exist anywhere in
the DVT ecosystem: complete `(state × input)` coverage as a proof strategy
rather than a sampling strategy.

A test suite that samples behavior is a statement about the inputs you thought
of. A test suite that covers every `(state, command)` pair is a statement about
the machine. Not "we tested many cases" — but "there are no untested cases."

The methodology is directly transferable. Any team that adopts it for their own
cluster state machine gains the same guarantee. The contribution is the
methodology; `faction` is the demonstration.

**The `no_std` guarantee — hardware diversity for CSM**

`faction` is `no_std + alloc`, enforced by CI. It compiles and runs identically
on a cloud VM and on constrained embedded hardware. For CSM solo stakers — many
of whom run on Raspberry Pi or home servers — this means the cluster lifecycle
primitive is available on their hardware without modification, directly
supporting hardware diversity as a decentralization goal.

---

### Current State — Phase 0 Is Delivered

Phase 0 — static cluster bootstrapping — is complete, published, and publicly
verifiable.

| Metric | Value |
|---|---|
| Productive LOC | 1,165 |
| Total tests | 275 |
| Code coverage | 100% |
| `(state, command)` pairs tested | All — explicitly |
| CRAP score (crappy functions) | 0 |
| Unsafe code | 0 — enforced by `#![forbid(unsafe_code)]` |
| `no_std` | Verified by CI |
| System test combinations | 10 |
| Spawn models validated | 3 — Task, Thread, Process |
| Transport protocols validated | 4 — Memory, Channels, TCP, gRPC |
| License | MIT |
| Published | https://crates.io/crates/faction |

Phase 0 is not a proposal. It is a shipped artifact. The quality bar it
demonstrates is the quality bar every subsequent phase will meet.

---

### Grant Scope — Phases 1–4

Each phase is a strict superset of the previous. Phase 0 tests pass unchanged at
Phase 4. No phase begins until the preceding phase has 100% `(state, command)`
coverage and CRAP score 0.

---

**Phase 1 — Dynamic operator joining** · 4–6 weeks · `faction` 0.4.x

An operator can request admission to an existing cluster at runtime. The machine
signals the join request; the caller (Charon, SSV client) decides the admission
policy; `faction` enforces it.

*DVT relevance:* Cluster operator replacement is one of the most operationally
complex events in a DVT cluster's lifecycle — a departing operator must be
replaced without disrupting ongoing signing duties. Today this is handled
ad hoc. Phase 1 gives it a formally tested state machine.

---

**Phase 2 — Failure detection** · 4–6 weeks · `faction` 0.5.x

SWIM-style probing: suspect → indirect probe → confirm offline or revive. The
machine emits probe commands; the caller executes them; quorum is re-evaluated
as liveness changes.

*DVT relevance:* Offline operator detection is one of the metrics the LNOSG
tracks explicitly. The current detection logic in Charon and SSV relies on
timeouts calibrated by intuition. Phase 2 replaces implicit timeout logic with a
provably correct liveness state machine where every failure response is covered
by the `(state, command)` transition matrix.

---

**Phase 3 — Safe operator addition** · 4–6 weeks · `faction` 0.6.x

A commit/abort reconfiguration protocol for cluster expansion. The machine
enforces a single-change-at-a-time structural invariant: a second addition
cannot be requested while one is in flight. At no point do two disjoint subsets
of the cluster each believe they hold quorum.

*DVT relevance:* Growing a cluster (e.g. from 4-of-6 to 5-of-7) while it is
actively signing requires that no signing duties are missed and no split-brain
condition is possible. Phase 3 makes this safe by construction — not by
convention, not by careful timing, but by a state machine where the unsafe
transitions do not exist.

---

**Phase 4 — Safe operator removal** · 4–6 weeks · `faction` 0.7.x

Symmetric to Phase 3, with harder invariants. The machine refuses removals that
would reduce the live set below quorum threshold. This is not a runtime check
that can be bypassed — it is a state transition that is structurally
unavailable when removal would make the cluster unable to sign.

*DVT relevance:* Operator exit — whether voluntary, due to failure, or due to
forced removal — requires that the remaining cluster can still reach quorum.
Phase 4 guarantees this at the type level. The connection to node operator
accountability mechanisms (where a misbehaving operator may need to be removed
from a live cluster) is direct and operationally significant.

---

### Impact

**For Obol and SSV teams:**
A formally tested reference implementation for the cluster lifecycle state
machine they both maintain in bespoke form today. A gap in the transition matrix
— a state the reference implementation handles that their code does not — is now
a findable, reproducible, fixable specification divergence rather than a latent
production incident.

**For Lido's decentralization:**
DVT cluster reliability is the load-bearing layer under Lido's decentralization
roadmap. More reliable bootstrapping and membership management means more node
operators can run DVT clusters with confidence, which means a broader and more
robust operator set. The four-fold growth in DVT validators over the past year
demonstrates the trajectory; `faction` ensures the formal correctness of the
infrastructure that trajectory depends on.

**For the Lido ecosystem broadly:**
A formally correct, MIT-licensed primitive that any team building DVT
infrastructure can import from crates.io today. The `(state × input)` testing
methodology is a transferable contribution — any protocol building a cluster
membership state machine can adopt it and gain the same completeness guarantee.

---

### About the Applicant

I am a senior software engineer with 15+ years of experience across embedded
systems, medical technology, finance, and defense. I hold an MSc in Computer
Engineering from La Sapienza, Rome (110/110 — highest distinction), and
currently contract at Hexagon (Leica Geosystems) as tech lead on an embedded
systems project. I hold ICP-ACC and iSAQB certifications.

My published crates are directly verifiable on crates.io and GitHub:

- **`faction`** — the subject of this proposal. Phase 0: 275 tests, 100%
  `(state, command)` coverage, 10-combination system test matrix across 3 spawn
  models and 4 transport protocols. CRAP score 0. Zero unsafe code. Published
  MIT.
- **`crap4rust`** — CRAP score quality gate used across all projects as a
  mandatory CI gate. Published MIT.
- **`fluxion`** — Rust stream aggregation with formal ordering guarantees.
  Published MIT.

I am also building **EtheRAM** — a distributed system (IBFT + Raft consensus,
EVM execution) running on real embedded hardware (Nucleo-F767ZI clusters). 
`faction` was extracted from EtheRAM when the cluster bootstrapping problem
became large enough to deserve its own formally specified solution. EtheRAM is
the validation environment that ensures `faction`'s design decisions are
grounded in the realities of a real distributed protocol.

I work independently. No team overhead, no coordination cost, no diffusion of
ownership. The person writing this proposal is the person who will implement
every line of code, write every test, and publish every crate version. The
execution record is public and verifiable today.

---

### Milestones and Verification

No payment is released until the preceding milestone's quality gates pass. Every
gate is publicly verifiable without self-reporting.

| Milestone | Deliverable | Estimated completion | Verification |
|---|---|---|---|
| Phase 1 complete | `faction` 0.4.x on crates.io | ~6 weeks post-start | Publish date · transition matrix · system/permutation tests |
| Phase 2 complete | `faction` 0.5.x on crates.io | ~12 weeks post-start | Publish date · transition matrix · system/permutation tests |
| Phase 3 complete | `faction` 0.6.x on crates.io | ~18 weeks post-start | Publish date · transition matrix · system/permutation tests |
| Phase 4 complete | `faction` 0.7.x on crates.io | ~26 weeks post-start | Publish date · transition matrix · system/permutation tests |

Quality gates at every phase boundary:
- 100% `(state, command)` coverage — transition matrix test file (public, executable)
- CRAP score 0 — `crap4rust` quality gate
- `#![forbid(unsafe_code)]` — enforced at compile time
- `no_std` — enforced at compile time

---

### Budget

**Total requested: $100,000 USDC**

| Item | Amount | Rationale |
|---|---|---|
| Engineering — Phases 1–4 | $90,000 | 24–26 weeks of principal-level distributed systems engineering at a sustainable Swiss independent contractor rate |
| LLM development tooling | $5,000 | DeepSeek, Anthropic, and Copilot API costs used throughout development |
| Contingency | $5,000 | Buffer for scope discoveries during formal correctness work — standard for this class of work |

At Swiss contracting market rates for principal-level distributed systems
engineering, $90,000 represents approximately 6–7 months of sustained part-time
work, or roughly 3 months of a senior engineer's fully-loaded cost at any of the
major DVT provider teams — in exchange for a formally correct cluster lifecycle
primitive that eliminates an entire class of DVT cluster incidents permanently
and benefits the entire ecosystem simultaneously.

Payment schedule, tied to milestone delivery:

| Milestone | Payment |
|---|---|
| Grant approval | $15,000 |
| Phase 1 complete — `faction` 0.4.x | $20,000 |
| Phase 2 complete — `faction` 0.5.x | $20,000 |
| Phase 3 complete — `faction` 0.6.x | $22,500 |
| Phase 4 complete — `faction` 0.7.x | $22,500 |

If the LEGO council prefers a smaller initial commitment, Phases 1–2 ($55,000)
are a natural standalone scope — they deliver dynamic operator joining and formal
failure detection, which are the most immediately operational contributions for
currently running DVT clusters, and can stand alone as a complete deliverable.
Phases 3–4 would then be a follow-on application after Phase 2 delivery
demonstrates execution quality.

---

### Conflict of Interest

`faction` was originally developed as part of EtheRAM, the applicant's personal
research project. EtheRAM uses `faction` as its cluster bootstrapping primitive.
This grant accelerates `faction`'s development in ways that also benefit EtheRAM.
EtheRAM is a non-commercial research project that generates no revenue. This is
disclosed as a transparency measure: the fact that `faction` is validated in a
real distributed protocol is what makes Phase 0's quality metrics credible rather
than abstract.

---

### Links

| Resource | URL |
|---|---|
| crates.io | https://crates.io/crates/faction |
| GitHub | https://github.com/umbgtt10/faction |
| ARCHITECTURE.md | https://github.com/umbgtt10/faction/blob/main/docs/ARCHITECTURE.md |
| ROADMAP.md | https://github.com/umbgtt10/faction/blob/main/docs/ROADMAP.md |
| Transition matrix | https://github.com/umbgtt10/faction/blob/main/core/tests/transition_matrix/state_transition_matrix_tests.rs |
| `crap4rust` | https://crates.io/crates/crap4rust |
| `fluxion` | https://crates.io/crates/fluxion-rx |

---

**ETH address to receive USDC:**
`0x6857fcdF0EE0bD6De569dD5dafB0374AE2901EaA`
