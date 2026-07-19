# Open Points

Status: **Open design questions** — raised while building Faction, not yet
decided, not yet scoped into a committed phase.

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

---

## Deferred / unwritten ADRs

- *(blocked on the quorum-change question above)* `Config` immutability —
  membership/quorum changes arrive as `Command`s, never as setters. No ADR until
  that question is decided.
- *(deferred to post-Phase-6)* `PeerId` genericization (currently `u64`).
- *(to write)* Testing-ladder ADR — the test tiers (unit / transition-matrix /
  property-based / system) and the gate that runs them. The ladder already
  exists; this is just writing it up.
