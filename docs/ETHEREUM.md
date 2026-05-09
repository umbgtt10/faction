# faction and Ethereum

> This document explains why `faction` matters specifically to the Ethereum ecosystem.
> Every technical claim links to the standard documentation.
> `faction` itself contains no Ethereum-specific code.

---

## The problem no Ethereum client team talks about

Ethereum has five production consensus clients: Lighthouse, Prysm, Teku, Nimbus, and
Reth. Each one is a serious engineering achievement. Each one implements Gasper, handles
attestation aggregation, manages fork choice, and produces blocks that the network
accepts.

Each one also solves — silently, independently, without formal specification — the
question that must be answered before any of that can happen:

**"Is my validator cluster actually ready to start consensus?"**

This is the bootstrapping problem. It is not glamorous. It does not appear in the
Gasper paper. It does not have a test suite in the consensus-spec-tests repository.
It is the part every client team wrote once, shipped, and hoped for the best.

The result: five separate implementations of the same problem. Five sets of timeouts
tuned by intuition. Five startup sequences that have never been tested against the
class of failures they are designed to survive — partial startup, Byzantine peers,
network partitions during initialization, validators that come online minutes apart.

When bootstrapping fails in production, it fails silently. The validator cluster
simply does not form. The error is indistinguishable from a network partition, a
misconfiguration, or a peer that is genuinely down. There is no formal specification
to debug against. There is no test that would have caught it. There is only the
on-call engineer at 2am with a log file.

`faction` is the formal, tested, protocol-agnostic primitive that ends this pattern.
Not for one client. For all of them.

---

## What `faction` eliminates

Every Ethereum client team that integrates `faction` stops writing and maintaining:

- Custom bootstrapping state machines with implicit, untested transitions
- Ad-hoc quorum detection with magic number thresholds
- Startup coordination logic that interacts badly with failure detection under load
- Validator readiness checks that were never tested against Byzantine inputs

They replace it with a formally specified [Mealy state machine](./ARCHITECTURE.md)
where every `(state, command)` pair is explicitly tested, every transition is
observable, and the correctness guarantees are stated up front and proven by
the [complete transition matrix](./core/tests/transition_matrix/state_transition_matrix_tests.rs).

The client team's engineers focus on what makes their client unique — their fork
choice optimizations, their attestation aggregation strategy, their sync protocol.
They do not spend engineering cycles on a problem that was already solved.

---

## The embedded validator opportunity

Ethereum's long-term decentralization depends on validator diversity. Geographic
diversity. Client diversity. And — increasingly — hardware diversity.

Today, running an Ethereum validator requires a machine capable of running a full
Go or Java runtime. This is not an accident. It is a consequence of the fact that
every production client is built for `std` environments. The `std` assumption is
invisible until you try to run a validator on a Raspberry Pi, a Jetson Orin, or an
STM32-class microcontroller — and discover that the assumption is baked into every
layer of the stack.

`faction` is `no_std + alloc`. This is not a convenience feature. It is a
deliberate constraint that means `faction` runs on the same binary on an
STM32-class embedded board and a 1000-node cloud cluster.

See the [design constraints](./ROADMAP.md#non-negotiable-constraints) for the
full rationale. See the [system test matrix](./README.md#validation-harness) for
the empirical proof: the same protocol code, validated across tasks, threads, and
real OS Processes, over Memory, Channels, TCP, and gRPC transports, with a timing
regime configurable enough to handle both nanosecond in-memory latency and
cross-process OS scheduling overhead.

The path to a validator that runs on constrained hardware — a Jetson Orin in a
data centre rack, a Raspberry Pi in a home, a hardened embedded board in a
physical installation — runs through `no_std`. `faction` is a foundational
primitive on that path.

---

## Dynamic membership and the validator lifecycle

The current `faction` implementation (Phase 0) solves static membership bootstrapping.
The [roadmap](./ROADMAP.md) extends it across six phases to full dynamic membership —
node joining, failure detection, single-node addition and removal, epoch-tracked
reconfiguration, and Byzantine-tolerant concurrent membership changes.

This matters to Ethereum in a concrete way: **validator set management is a
membership problem**.

A validator joining the active set, exiting voluntarily, being slashed and removed,
being detected as offline and suspected — these are all membership events. Today,
every client team handles them with bespoke logic that interacts with their specific
consensus implementation in ways that are difficult to reason about formally.

When `faction` reaches Phase 6, the complete dynamic membership lifecycle is handled
by a formally tested, protocol-agnostic primitive. A client team integrating `faction`
at that point can reason about validator lifecycle events in terms of a specified state
machine with proven invariants, rather than ad-hoc logic scattered across their
codebase.

The phases build incrementally. Each phase ships as a published crate version.
Each phase's correctness is established before the next begins.
See the [complete roadmap](./ROADMAP.md).

---

## The testing methodology as a contribution in its own right

The Ethereum consensus-spec-tests repository is the ecosystem's benchmark for
client correctness. It defines what a correct Gasper implementation looks like and
provides test vectors that every client must pass.

It does not define what correct bootstrapping looks like. It does not provide test
vectors for the validator lifecycle. It does not test the class of failures that
`faction` is designed to handle.

`faction` introduces a testing methodology that fills this gap:
**complete `(state × input)` coverage** as a proof strategy rather than a sampling
strategy.

The distinction matters. A test suite that samples behavior — here is a happy path,
here is one error case, here is one Byzantine input — is a statement about the inputs
you thought of. A test suite that covers every `(state, command)` pair is a statement
about the machine. Not "we tested many cases." But "there are no untested cases."

The [transition matrix](./core/tests/transition_matrix/state_transition_matrix_tests.rs)
is that statement made executable.

This methodology is applicable beyond `faction`. Any Ethereum client team that adopts
it for their own state machines gains the same guarantee: not a coverage percentage,
but a proof. The methodology is the contribution. `faction` is the demonstration.

---

## Relevance to post-quantum Ethereum

The Ethereum Foundation's post-quantum research agenda targets the replacement of
BLS12-381 signatures with quantum-resistant alternatives — most likely ML-DSA
(CRYSTALS-Dilithium) for attestation signing and aggregation.

This transition touches every layer of the consensus stack. It changes the signature
scheme, the aggregation protocol, the networking layer, and — critically — the
bootstrapping and membership management layer. A validator cluster using ML-DSA
attestations has different bootstrapping semantics than one using BLS: the aggregation
structure changes, the quorum evidence changes, and the membership signals that
`faction` tracks must be adapted accordingly.

`faction`'s protocol-agnostic design means this adaptation is localized. The state
machine's invariants — the two-phase design, the complete `(state × input)` coverage,
the observer model — are independent of the signature scheme. The caller provides
the membership signals. The machine tracks them. Whether those signals carry BLS
proofs or ML-DSA signatures is the caller's concern, not `faction`'s.

A post-quantum Ethereum still needs to answer the bootstrapping question before
consensus begins. `faction` answers it the same way regardless of what cryptography
the rest of the stack uses.

---

## What a grant enables

`faction` Phase 0 is complete, published, and MIT-licensed. The bootstrapping
primitive exists. The testing methodology is demonstrated. The `no_std` claim is
verified across 15 spawn/transport combinations.

What a grant from the Ethereum Foundation enables is the completion of the full
lifecycle management roadmap:

| Phase | What it delivers | Ethereum relevance |
|---|---|---|
| 1 | Dynamic joining | Validator onboarding without restart |
| 2 | Failure detection (SWIM) | Offline validator detection with formal guarantees |
| 3 | Single-node addition | Safe validator set expansion |
| 4 | Single-node removal | Safe validator exit and slashing response |
| 5 | Epochs and rejoin | Split-brain prevention across restarts |
| 6 | Concurrent changes | High-throughput validator set management under load |

**This grant funds Phases 1 through 4.** These four phases deliver the validator
lifecycle events most critical to Ethereum today: onboarding, liveness monitoring,
safe expansion, and safe exit. They form a complete, independently useful primitive
at a natural stopping point — addition and removal together close the loop on
membership correctness.

Phases 5 and 6 — epoch management and concurrent changes — are the subject of a
subsequent grant application. The academic paper targeting EuroSys 2027, written
in parallel with Phases 4 through 6, provides peer-reviewed validation of the
methodology and strengthens the case for continued funding.

Each phase is independently useful. Each phase ships before the next begins.
Each phase extends the same formally tested foundation — no phase introduces
untested behavior.

Full timeline and milestone detail in [ROADMAP.md](./ROADMAP.md).

---

## Why now

The Ethereum ecosystem is at an inflection point on three dimensions simultaneously:

**Client diversity.** Five production clients is good. Five clients each maintaining
their own untested bootstrapping logic is a latent correctness risk that compounds as
the network grows and validator set management becomes more dynamic.

**Hardware diversity.** The path to embedded validators is open. The `no_std`
primitive that would make it practical exists. What is missing is the complete
lifecycle management stack built on that foundation.

**Post-quantum transition.** The signature scheme is changing. The window to establish
formally correct, protocol-agnostic lifecycle management primitives before that
transition — while the stack is already being rearchitected — is narrow.

`faction` addresses all three. The foundation is already built. The grant funds
the completion.

---

## Standard documentation

All technical claims in this document are substantiated by the standard documentation,
which contains no Ethereum-specific content:

- [README.md](./README.md) — what `faction` does, how to use it, project status
- [ARCHITECTURE.md](./docs/ARCHITECTURE.md) — complete technical specification
- [ROADMAP.md](./docs/ROADMAP.md) — phased development plan and hard rules
- [transition_matrix_tests.rs](./core/tests/transition_matrix/state_transition_matrix_tests.rs) — the complete `(state × input)` proof
