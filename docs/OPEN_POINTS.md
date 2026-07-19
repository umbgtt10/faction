# Open Points

Status: **The Phase-0 hardening bug is DONE.** The concluded-node silent-sink
family (formerly §1–§4, §6, §7) was fixed on the `fix/terminal-state-sinks`
branch and re-baselined into Phase 0's own test suite. What remains here is a
short record of that fix, plus the genuinely-open questions.

Companion to `ROADMAP.md`. Items move into `ROADMAP.md` (or a phase's own spec)
once decided; until then they stay here rather than clutter the roadmap with
unsettled debate. Settled architectural decisions live in `docs/ADRs/`.

---

## Done — the concluded-node silent-sink bug (was §1–§4, §6, §7)

A node that reached a terminal state stopped helping its peers. Two shapes, one
root cause: terminal states were hard sinks — `accept()` returned `false`, so a
concluded node neither re-advertised to help others nor stayed receptive to
recover itself.

- **`Bootstrapped` went silent.** A node that reached quorum cancelled its
  retries and never re-advertised, so a peer that missed the original readiness
  broadcast (a dropped message, or a fast concluder) was stranded forever.
  First seen on real hardware in two independent consumers (etheram-embassy,
  and etheram-ibft's `Bootstrapper` — fixed downstream in `8fba86c`); then
  reproduced in-crate by the `(size, quorum)` convergence sweep at `(2, 2)`.
- **`DeadlineExpired` was a dead end.** A deadline before quorum drove the node
  into a terminal `TimedOut` state it could never leave, even when the readies
  it needed arrived late.

**Fix (both phases landed, gates green):**

- `Bootstrapped` now accepts `ParticipationObserved` and emits
  `AcknowledgeRejoin { peer_id }` — the consumer re-advertises readiness to the
  still-pinging peer. It stays terminal but is no longer a silent sink.
- `DeadlineExpired` is non-terminal: `Pinging`/`Collecting` record
  `DeadlineMissed { confirmed_count }` and stay in place, keeping their retries.
  The internal `TimedOut` state and `Conclusion::TimedOut` are removed;
  `PeerState::TimedOut` survives as a derived view flag off a new
  `ClusterView::deadline_missed`. `Bootstrapped` is the only terminal state.
- Two adjacent finds fixed along the way: single-node clusters now bootstrap
  (`Initial` accepts `LocalParticipationCompleted`), and a `protocol-validation`
  harness routing bug (`step_transport_node` mis-attributed outputs).

Acceptance tests: `dropped_ready` `(2, 2)` (Bootstrapped sink) and
`late_arrival::cluster_recovers_after_deadline_via_late_readiness` (TimedOut
sink); the `deadline_expired` sweep was re-baselined from "dead-ends in
TimedOut" to "stays receptive". Recorded in `CHANGELOG.md` under `[Unreleased]`
and in `docs/ADRs/P1-ADR-TerminalStatesAreNotSinks.md`.

**Follow-ups (outside this crate):**

- The etheram-ibft `Bootstrapper` workaround (`8fba86c`) can now collapse
  toward the thin-adapter target: `locally_completed` disappears (Faction knows
  it from its own state), and the ack branch maps `AcknowledgeRejoin` → wire
  reply. etheram-raft wires the two outcomes from the start and never grows the
  workaround. That the workaround collapses to a forwarding shim is the proof
  the Faction/consumer boundary was drawn in the right place.
- These are breaking changes (new outcomes, removed `Conclusion::TimedOut`,
  `Bootstrapped` admits `ParticipationObserved`) — they release as **0.4.0**.

---

## Open — should Faction allow the quorum size to change?

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

---

## Open — deferred / unwritten ADRs

- *(blocked on the quorum-change question above)* `Config` immutability —
  membership/quorum changes arrive as `Command`s, never as setters. No ADR until
  that question is decided.
- *(deferred to post-Phase-6)* `PeerId` genericization (currently `u64`).
- *(to write)* Testing-ladder ADR — the test tiers (unit / transition-matrix /
  property-based / system) and the gate that runs them. The ladder already
  exists; this is just writing it up.
