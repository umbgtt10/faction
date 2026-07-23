# Ethereum Foundation Grant Application
## `faction` — Formal Validator Lifecycle Management for the Ethereum Ecosystem

**Applicant:** Umberto Gotti
**Category:** Core Protocol Research  
**Amount requested:** $100,000 USD  
**Grant scope:** Phases 1–4 of the `faction` roadmap  
**Date:** May 2026  

---

## Project summary

`faction` is a formally specified, completely tested, protocol-agnostic Mealy state
machine for distributed cluster bootstrapping and validator lifecycle management.
It is `no_std + alloc`, contains zero unsafe code, and runs without modification
on an STM32-class microcontroller and a 1000-node cloud cluster.

Phase 0 — static membership bootstrapping — is complete, published on crates.io
under the MIT license, and verified across 10 spawn/transport combinations including
real OS processes communicating over real TCP and gRPC connections.

This grant funds Phases 1 through 4: dynamic joining, failure detection, single-node
addition, and single-node removal. These four phases deliver a complete, formally
tested implementation of the validator lifecycle events most critical to Ethereum.

The deliverable is not a paper that describes a correct implementation. It is the
correct implementation — formally specified, completely tested, and shipping on
crates.io at each phase boundary.

---

## The problem

Ethereum has five production consensus clients: Lighthouse, Prysm, Teku, Nimbus,
and Reth. Each implements Gasper, handles attestation aggregation, manages fork
choice, and produces blocks the network accepts.

Each also solves — silently, independently, without formal specification — the
question that must be answered before any of that can happen:

**"Is my validator cluster actually ready to start consensus?"**

This is the bootstrapping problem. It does not appear in the Gasper paper. It has
no test suite in the consensus-spec-tests repository. It is the part every client
team wrote once, shipped, and hoped for the best.

The result is five separate implementations of the same problem. Five sets of
timeouts tuned by intuition. Five startup sequences never tested against the class
of failures they are designed to survive: partial startup, Byzantine peers, network
partitions during initialization, validators that come online minutes apart.

When bootstrapping fails in production, it fails silently. The validator cluster
does not form. The error is indistinguishable from a network partition, a
misconfiguration, or a genuinely down peer. There is no formal specification to
debug against. There is no test that would have caught it.

The same pattern extends to the full validator lifecycle. Validator joining,
voluntary exit, slashing response, offline detection — these are all membership
events. Every client team handles them with bespoke logic coupled to their specific
consensus implementation in ways that are difficult to reason about formally and
impossible to test in isolation.

This is not a criticism of any client team. It is a gap in the ecosystem's
infrastructure. The primitive that should exist — a formally specified, completely
tested, protocol-agnostic membership state machine — has never been built.

Until now.

---

## The solution

`faction` is that primitive.

It implements a pure Mealy state machine: `output = F(state, input)`. No side
effects inside the machine. No network I/O. No consensus algorithm. No opinion on
what transport the caller uses or what "ready" means. The caller owns the network.
`faction` owns the state transitions.

**Why a Mealy machine?**

The Mealy model was chosen for four reasons. First, complete testability: because
`output = F(state, input)`, the entire behavior of the machine is captured by the
set of `(state, command)` pairs. This set is finite and enumerable. The test suite
covers every pair explicitly — not as a sample, but as a proof. Second, deterministic
replay: any production incident is reproducible by replaying the input log against
the initial state. Third, clean separation of concerns: the machine computes, the
caller acts. Fourth, formal verifiability: each TLA+ action maps directly to one
machine transition.

**The testing methodology as a research contribution**

The Ethereum consensus-spec-tests repository defines what a correct Gasper
implementation looks like. It does not define what correct bootstrapping looks
like. It does not provide test vectors for the validator lifecycle.

`faction` introduces a testing methodology that fills this gap: complete
`(state × input)` coverage as a proof strategy rather than a sampling strategy.

A test suite that samples behavior is a statement about the inputs you thought of.
A test suite that covers every `(state, command)` pair is a statement about the
machine. Not "we tested many cases." But "there are no untested cases."

The complete transition matrix is not a document. It is executable code:
[state_transition_matrix_tests.rs](https://github.com/umbgtt10/faction/blob/main/core/tests/transition_matrix/state_transition_matrix_tests.rs).
Every combination — named, tested, asserted.

This methodology is applicable beyond `faction`. Any Ethereum client team that
adopts it for their own state machines gains the same guarantee. The methodology
is the contribution. `faction` is the demonstration.

**What makes the deliverable unusual**

Most core protocol research delivers a paper or a formal model. Reproducing the results
requires rebuilding the implementation from scratch. `faction` delivers the research
contribution and the production artifact as the same thing: a published, MIT-licensed
crate that any client team can import today.

Phase 0 is on crates.io right now. The research is already deployed.

**The `no_std` constraint and embedded validators**

Ethereum's long-term decentralization depends on hardware diversity. Every
production client is built for `std` environments. This assumption is invisible
until you try to run a validator on a Raspberry Pi, a Jetson Orin, or an
STM32-class microcontroller — and discover it is baked into every layer of the stack.

`faction` is `no_std + alloc`. This is not a convenience feature. It is a
deliberate constraint verified empirically: the same protocol code validated across
tasks, threads, and real OS processes, over Memory, Channels, TCP, and gRPC
transports, with timing regimes calibrated for both nanosecond in-memory latency
and cross-process OS scheduling overhead — 10 combinations in total, all green.

The path to a validator running on constrained hardware runs through `no_std`.
`faction` is a foundational primitive on that path.

**Post-quantum alignment**

`faction`'s protocol-agnostic design means its invariants are independent of the
signature scheme. Whether the membership signals it tracks carry BLS proofs or
ML-DSA signatures is the caller's concern, not `faction`'s. A post-quantum
Ethereum still needs to answer the bootstrapping question before consensus begins.
`faction` answers it the same way regardless of what cryptography the rest of the
stack uses.

---

## Current state

Phase 0 is complete. The evidence is public and verifiable.

| Metric | Value |
|---|---|
| Productive LOC | 1,165 |
| Total tests | 275 |
| Code coverage | 100% |
| `(state, command)` pairs tested | All — explicitly |
| Crappy functions (CRAP score) | 0 |
| Unsafe code | 0 — enforced by `#![forbid(unsafe_code)]` |
| `no_std` | Verified |
| System test combinations | 10 |
| Spawn models validated | 3 — Task, Thread, Process |
| Transport protocols validated | 4 — Memory, Channels, TCP, gRPC |
| Published | crates.io (MIT) |

This is not a proposal. Phase 0 is a delivered artifact that demonstrates both
the execution velocity and the quality bar that every subsequent phase will meet.

---

## Grant scope: Phases 1–4

This grant funds four phases. Each phase is a strict superset of the previous.
Phase 0 tests pass unchanged at Phase 4. No phase begins until the previous phase
has 100% `(state, command)` coverage and a CRAP score of 0.

**Phase 1 — Dynamic joining** (4–6 weeks)

The peer set is no longer fixed at construction. A validator can request admission
to the cluster at runtime. `faction` signals the request to the caller, who decides
the admission policy. The machine enforces the decision.

*Ethereum relevance:* validator onboarding without cluster restart.

**Phase 2 — Failure detection** (4–6 weeks)

SWIM-style probing: suspect → indirect probe → confirm or revive. The machine emits
probe commands. The caller executes them. Quorum is re-evaluated as liveness changes.

*Ethereum relevance:* offline validator detection with formal, testable guarantees —
not a timeout tuned by intuition.

**Phase 3 — Single-node addition** (4–6 weeks)

A commit/abort reconfiguration protocol for safe membership expansion. The machine
enforces a structural single-change-at-a-time invariant: it is impossible to request
a second addition while one is in flight. At no point do two disjoint subsets of
nodes each believe they form a valid quorum.

*Ethereum relevance:* safe validator set expansion with split-brain prevention by
construction.

**Phase 4 — Single-node removal** (4–6 weeks)

Symmetric to Phase 3, with harder invariants. The machine refuses removals that
would drop the live set below quorum threshold. This is not a runtime check — it
is a state transition that is structurally unavailable when the live set would be
insufficient.

*Ethereum relevance:* safe validator exit and slashing response with quorum
preservation guaranteed by the type system, not by convention.

Phases 1 through 4 form a complete, independently useful primitive at a natural
stopping point: addition and removal together close the loop on membership
correctness. Phases 5 and 6 — epochs and rejoin, and concurrent changes — are the
subject of a subsequent grant application, justified by the demonstrated delivery
of Phases 1 through 4.

---

## Impact

**For Ethereum client teams:**

Every client team that integrates `faction` stops writing and maintaining custom
bootstrapping state machines with implicit, untested transitions. They replace bespoke
validator lifecycle logic with a formally specified primitive where every transition
is observable, every `(state, command)` pair is tested, and the correctness
guarantees are stated up front and proven in executable code.

Their engineers focus on what makes their client unique. They do not spend cycles
on a problem that is already solved.

**For the ecosystem:**

Five clients sharing one formally tested lifecycle primitive is a stronger security
posture than five clients maintaining five separate untested implementations. A bug
found and fixed in `faction` is fixed for all of them simultaneously.

**For hardware diversity:**

The `no_std` foundation makes embedded validators possible. Not as a future
ambition — as a direct consequence of the architectural constraint that is already
enforced and already verified.

**For the testing methodology:**

The `(state × input)` proof strategy, demonstrated on a non-trivial distributed
systems primitive, is a transferable contribution. Any team that adopts it gains
the same guarantee. The methodology outlives any single crate.

---

## Budget

**Total requested: $100,000 USD**

| Item | Amount | Rationale |
|---|---|---|
| Engineering — Phases 1–4 | $90,000 | 24–26 weeks of principal-level distributed systems engineering at a sustainable independent rate |
| LLM Tokens | $5,000 | Anthropic, OpenAI and Copilot subscriptions |
| Contingency | $5,000 | Buffer for scope discoveries during implementation — standard on formal correctness work |

At Swiss contracting market rates for principal-level distributed systems
engineering, $90,000 represents approximately 6–7 months of part-time work. This
grant asks for less than 3 months of a senior engineer's fully-loaded cost at any
of the five major client teams, in exchange for a formally correct primitive that
eliminates the bootstrapping and lifecycle management problem for all of them
permanently.

---

## Milestones and verification

Each phase produces a verifiable, public artifact. No milestone payment is claimed
until the artifact is published and the quality gates pass.

| Milestone | Deliverable | Verifiable by |
|---|---|---|
| Phase 1 complete | `faction` 0.4.x on crates.io | crates.io publish date + test count + coverage report |
| Phase 2 complete | `faction` 0.5.x on crates.io | crates.io publish date + test count + coverage report |
| Phase 3 complete | `faction` 0.6.x on crates.io | crates.io publish date + test count + coverage report |
| Phase 4 complete | `faction` 0.7.x on crates.io | crates.io publish date + test count + coverage report |

Every published version carries:
- 100% `(state, command)` coverage — verified by the transition matrix test file
- CRAP score 0 — verified by the `crap4rust` quality gate
- `#![forbid(unsafe_code)]` — enforced at compile time
- `no_std` — verified by the CI `no_std` check script

The quality gates are not self-reported. They are publicly verifiable from the
repository and the published crate.

---

## About the applicant

I am a senior software engineer with 15+ years of experience across medical
technology, finance, and defense/embedded systems. I hold an MSc in Computer
Engineering from La Sapienza, Rome (110/110 — highest distinction).

I have worked at UBS and in a SAFe environment at Roche, and I currently contract
at Hexagon on embedded systems involving Jetson-class hardware and Yocto-based
Linux. I hold ICP-ACC and iSAQB certifications.

My published crates demonstrate both the technical depth and the execution velocity
relevant to this grant:

- **`faction`** — the subject of this application. Phase 0: 275 tests, 100%
  coverage, 10-combination system test matrix, published MIT.
- **`fluxion`** — Rust-idiomatic implementation of composite Rx extensions for stream aggregation
  with ordering guarantee, friendly fluent API, bulletproof testing, runtime abstraction and top-notch documentation, published MIT.
- **`crap4rust`** — a Rust implementation of the CRAP (Change Risk Anti-Patterns)
  quality metric, used as a quality gate across all my projects.

Beyond the tooling portfolio, I am building **EtheRAM** — a minimal
Ethereum-like blockchain (IBFT + Raft consensus, EVM execution, gas metering)
running on real embedded hardware (Nucleo-F767ZI). `faction` was extracted from EtheRAM as a
standalone primitive when it became clear the bootstrapping problem deserved its
own formally specified solution.

EtheRAM is the validation environment that ensures `faction`'s design decisions
are grounded in the realities of a real distributed protocol, not an abstract
exercise.

I work independently. There is no team overhead, no coordination cost, and no
diffusion of ownership. The person writing this application is the person who
will implement every line of code, write every test, and publish every crate
version. The execution track record is public and verifiable on crates.io and
GitHub today.

---

## Links

| Resource | URL |
|---|---|
| crates.io | https://crates.io/crates/faction |
| GitHub | https://github.com/umbgtt10/faction |
| ETHEREUM.md | https://github.com/umbgtt10/faction/blob/main/docs/ETHEREUM.md |
| ARCHITECTURE.md | https://github.com/umbgtt10/faction/blob/main/docs/ARCHITECTURE.md |
| ROADMAP.md | https://github.com/umbgtt10/faction/blob/main/docs/ROADMAP.md |
| Transition matrix | https://github.com/umbgtt10/faction/blob/main/core/tests/transition_matrix/state_transition_matrix_tests.rs |
| `crap4rust` | https://crates.io/crates/crap4rust |
| `fluxion` | https://crates.io/crates/fluxion-rx |
