# Open Points

Status: **Phase 1 (dynamic joining) landed** across the full spawn/transport matrix —
see `CHANGELOG.md` for what shipped and `docs/ADRs/` for the decisions. Everything
implemented, decided, or captured in an ADR has moved out of this file; only the
genuinely-open items remain below.

Companion to `ROADMAP.md`. Items move into `ROADMAP.md` (or a phase's own spec)
once decided; until then they stay here rather than clutter the roadmap with
unsettled debate. Settled architectural decisions live in `docs/ADRs/`; shipped
changes live in `CHANGELOG.md`.

---

## Should Faction allow the quorum size to change?

Not a bare setter — that would reintroduce, through a side door, the exact
hazard Phase 3 already exists to prevent. If threshold can be swapped on one
node without coordination, two disjoint subsets of the cluster can each end up
running a different threshold, each independently convinced it holds quorum — a
safety violation, not a liveness hiccup. Phase 3's own stated invariant ("at no
point do two disjoint subsets of nodes each believe they form a valid quorum,"
enforced by making single-change-at-a-time structurally impossible) is precisely
the machinery this needs, not something to bypass by adding mutability to
`Config` in Phase 0.

**Refinement:** the change should be exposed through the same mechanism as
everything else — a genuine new `Command` variant (something like
`Command::QuorumPolicyChanged { new_policy: QuorumPolicy }`), evaluated and
scoped per state through the normal `accept()`/`step()` gating, inheriting
exhaustive `(state, command)` coverage and `Observer` routing like every other
command, rather than an out-of-band setter that bypasses the state machine's own
input gating entirely. That's a necessary foundation and a real improvement over
a bare setter — but it doesn't by itself solve the cross-node coordination
problem. Being a well-formed, per-state-gated command makes the *local* decision
explicit and testable; it's still Phase 3/4's commit/abort sequencing that has
to guarantee the *cross-node* agreement about when the new value takes effect.
The two are complementary, not substitutes for each other.

Faction still never computes what the new threshold *should* be, under either
version — that stays the protocol's fault-model-driven formula, recomputed and
injected by the consumer, same as at construction.

The postponed **`join-raises-quorum`** system test (Phase 1 scenario 5) awaits this
command: it presupposes the caller can inject a larger threshold on admission — exactly
the uncoordinated threshold swap deferred here — so it returns only once the
quorum-change command lands. That admission alone never moves the threshold as the set
grows is already covered at the core level by
`join_tests.rs::admission_does_not_change_the_quorum_threshold`.

---

## Future consumers (no assumptions)

The general win is a protocol-agnostic, exhaustively-covered **admission gate**:
any system where a node comes online at runtime and someone must answer "is this
peer allowed to participate, and are we ready?" can borrow the gatekeeping state
machine while keeping *policy* (who is eligible) in the protocol. Gasper
specifically: L1 activation/exit is the beacon chain's stake+queue logic, *not* a
Faction concern; the honest fit is one layer out — distributed-validator
middleware (Obol/SSV-style DV clusters) where a newly-activated validator's client
joins a local DV cluster. Other candidates to evaluate later (not commitments):
gossip/SWIM membership layers, sidecar/sharded clusters, any BFT SMR wanting a
bootstrap+join gate. Each such consumer should contribute back a published sample
integration (the ROADMAP publication checklist already asks for one).

---

## Robustness under adversarial delivery (pre-Phase 2)

The whole transport matrix — InMemory, Channels, TCP, gRPC — is **reliable and
ordered**: everything is delivered, in order, exactly once. That proves faction is
*transport-agnostic*; it has never tested *delivery quality* — loss, reordering,
duplication, partitions.

**A new transport (WebSocket / UDP)?** Not for correctness. WebSocket is another
reliable framed transport (gRPC's shape) — breadth, not a new failure mode. UDP looks
tempting for loss/reorder, but localhost UDP essentially never drops, so it would not
exercise its distinguishing property without artificial dropping — at which point it
*is* the fault-injection work below. Adding a transport is a breadth call
(grants/marketing), not a coverage one.

**Fault injection — the real gap.** A harness-level `FaultyTransport` decorator
(built now; exercised as the Phase-2 hardening assessment) wraps any transport and
misbehaves per a seeded policy. It lives in the harness, **not** in `core` (pure
Mealy, no I/O — it cannot "drop") or `protocol` (a translator, not a network). The
taxonomy, each row a claim faction already makes:

| Fault | Probes the claim that… |
|---|---|
| Loss | `RetryPing`/`RetryReady` recover dropped signals (liveness under loss) |
| Duplication | set-accumulation + `Duplicate*Ignored` dedup (safety under dup) |
| Partition (+ heal) | a cut sub-quorum → `TimedOut` → converges on heal (partition tolerance + self-heal) |
| Delay / jitter | the deadline-vs-latency boundary; `TimedOut`-then-recover |
| Reordering | commutative set accumulation (order-independence) |
| Asymmetric / one-way | convergence survives half-open links |
| Selective by type | each retry path is individually sufficient (dropping `Bootstrapped` announces is harmless — they map to `Probe`) |

Out of scope: **corruption / mutation** is a *lying peer* → Byzantine (Phase 5), and
the wire decode already turns garbage into loss; **throttle / bandwidth** — faction
cares about delivery, not throughput.

**Hardening hypothesis.** Faction is already designed for this — retries, idempotent
sets, and a non-terminal `TimedOut` that self-heals — so the expected Phase-2 verdict
is *no core hardening*: the real knob is operational (the deadline / freshness margin
tuned against the loss rate, which per faction's design is the caller's, not the
machine's). The outcome that *would* mean hardening is a **permanent stall below
100 % loss** — a convergence-critical signal that turns out not to be retried. That is
what the assessment hunts for.

**Fault logging (deferred with the wiring).** When `FaultyTransport` is wired into
`ClusterBuilder`, a fired fault is recorded as one more line in the same per-node log the
observers already write (`{"event":"fault","fault":…,"to":…,"message":…}`, sharing that
node's writer) — so faults land inline in the per-node files and the merged
`consolidated.jsonl` timeline, never a separate file. Deferred to the Phase-2 wiring.

---

## Deferred

- `PeerId` genericization (currently `u64`) — deferred to post-Phase-6.
