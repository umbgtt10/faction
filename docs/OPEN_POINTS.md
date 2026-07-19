# Open Points

Status: **PARTIALLY DECIDED — collected 2026-07-17; §1–§4 committed as a Phase 0 bug fix, and persistency-free named as an invariant, on 2026-07-19 (see §9, §10)**

Companion to `ROADMAP.md`. This document holds unresolved design questions
and considerations raised while investigating Phase 0's known
rejoin-acknowledgment limitation — things under active discussion, not yet
decided, not yet scoped into a committed phase. Items move into
`ROADMAP.md` (or into a phase's own spec) once they're actually decided;
until then they stay here rather than clutter the roadmap with unsettled
debate.

**2026-07-19 update:** §1–§4 are now decided — they are treated as a **bug
in Phase 0**, fixed in place before any Phase 1 work begins. This is *not* a
new roadmap phase; the ROADMAP is unchanged. §9 records the decision, the
concrete consumer payoff, and how the fix re-baselines Phase 0's own tests.
The source-of-truth boundary for Phase 3–5 membership is now decided too —
Faction is **persistency-free** (§10), which makes the consumer's committed
log the sole membership authority and turns the Raft/IBFT integration
asymmetry into "govern, don't replace" rather than a collision.

---

## 1. Known limitation — discovered via integration testing (2026-07-16)

A statically-known member that restarts mid-cluster-lifetime re-enters
through the same `Pinging`/`Collecting` flow as a brand-new cold boot.
Already-`Bootstrapped` peers never acknowledge its discovery pings, so it
always rides out the full `DeadlineExpired` timeout before its buffered
messages get processed — even though it was receiving valid traffic from
every peer the whole time. This is distinct from Phase 1 (a genuinely new
peer joining) and Phase 5 (a previously dynamically-removed peer
rejoining) — the restarting node was never removed from the static set,
it just temporarily went away. Not caught by Phase 0's own `(state,
command)` test suite, which doesn't model a long-running cluster with a
mid-lifetime member restart. Found and fully traced (two independently
cross-checked clocks, confirmed against `deadline` down to the second) via
a real-hardware integration test in a downstream consumer project. Needs a
fast reconnect path for known peers — **now treated as a Phase 0 bug to fix (§9).**

**Confirmed a second time, independently (2026-07-19).** The same class of
gap reproduced on real hardware in a *different* downstream consumer,
etheram-ibft's `Bootstrapper`, in a different shape: a node that had
broadcast its own readiness but not yet observed quorum of peers' readiness
("locally complete, not yet concluded") was silently dropping rejoin pings
from a restarting peer and never re-announcing its own readiness — a ~3
minute stall until the stale deadline timer fired. Fixed downstream
(`locally_completed` flag + rejoin-ack branch + readiness rebroadcast,
etheram-ibft `8fba86c`). That fix is a consumer-side reimplementation of
accounting §6 assigns to Faction — exactly the duplication this crate
exists to prevent (same failure mode as the `Some(node_count)` quorum bug
landing independently in both consumers). §9 records the plan to pull it
back into Faction.

---

## 2. `TimedOut` shares `Bootstrapped`'s original dead end

`TimedOut::accept()` (`core/src/states/timed_out.rs`) returns `false`
unconditionally — the same pattern `Bootstrapped` had before its rejoin-ack
extension. So once `DeadlineExpired` fires before quorum in `Pinging` or
`Collecting`, the resulting `TimedOut` state locks out any further
`ParticipationObserved` or `ReadyObserved`, permanently. Any fix for the
rejoin gap that only touches `Bootstrapped` is incomplete — `TimedOut` needs
the same treatment, or to stop existing as a separate terminal state (see
section 4).

---

## 3. What should Faction do with a consumer-issued `DeadlineExpired`?

Resolved direction, not yet implemented: **record the fact, stay exactly
where it was.** A state transition on `DeadlineExpired` is itself the policy
decision ("missing this means stop") — and that decision isn't Faction's to
make (see section 6). Concretely, in `Pinging`/`Collecting`:

```rust
Command::DeadlineExpired => (
    vec![Outcome::DeadlineMissed { confirmed_count: self.pinging_peers.len() }],
    Box::new(Self { /* unchanged fields */ }),
),
```

- **Not `Outcome::Concluded { mode: Conclusion::TimedOut }`.** "Concluded"
  means final; reusing it here would just relocate the same false claim
  from the state machine into the outcome vocabulary. Needs a genuinely new,
  non-terminal outcome (`DeadlineMissed` or similar), leaving `Concluded`
  meaning only `Bootstrapped` from here on.
- **Carries the current confirmed count**, not just a bare signal — still
  pure accounting (Faction already has this data), but the difference
  between "missed deadline, no idea how bad" and "missed deadline, 3 of 4
  confirmed" is real diagnostic value at zero cost to the boundary.
- **Repeatable, deliberately.** A consumer may fire `DeadlineExpired` again
  later as a recurring check-in rather than a one-shot cutoff. Faction
  should answer again, honestly, with whatever the current count is — no
  deduplication, no "already told you that."
- **`Bootstrapped` still rejects it** — a stale deadline timer firing after
  bootstrap already completed is a caller-side bookkeeping artifact, not a
  new fact, consistent with how stale timers are already handled elsewhere
  in this ecosystem (logged and ignored, not treated as an error).
  `Initial` stays invalid too — a deadline firing before pinging has even
  started means the caller scheduled it wrong.

---

## 4. Should `TimedOut` be removed as a separate state entirely?

Leaning yes, not yet decided. If `TimedOut` stops being a dead end, its
correct behavior for `ParticipationObserved`/`ReadyObserved` would have to
be *identical* to whichever of `Pinging`/`Collecting` produced it — there's
no other correct behavior once "keep trying toward quorum" is the rule.
Keeping it as a separate struct at that point means duplicating logic that
already exists elsewhere, working against the crate's own complexity
rules. The cleaner shape: no separate `TimedOut` *state* at all — just the
`deadline_missed` fact from section 2, tracked within whichever state
actually produced it.

Distinction worth preserving: `PeerState` (the public, `ClusterView`-facing
enum) is separate from the internal state structs, and already has its own
`TimedOut` variant independent of the `impl State for TimedOut` struct.
`PeerState::TimedOut` could keep being reported as a derived value off the
`deadline_missed` flag even after the internal struct disappears — only the
dead-end *behavior* needs to go, not the external signal.

**Not yet verified:** whether the internal state structs (`Pinging`,
`Collecting`, `Bootstrapped`, `TimedOut`) are `pub` in a way that makes them
part of the crate's semver surface, versus purely internal to the
`Box<dyn State>` machinery. Faction is already published on crates.io, so
this determines whether removing `TimedOut` is a clean internal refactor or
a breaking change requiring a version bump.

---

## 5. Should the deadline/timeout period be `Option<Duration>` at Faction's core?

Checked directly: **there's nothing to make optional.** `Config`
(`core/src/config.rs`) carries only `peer_id`, `peers`, `quorum_policy` — no
duration field anywhere, ever. Faction never owned "how long to wait";
`DeadlineExpired` is just a command a consumer chooses to send or not send,
entirely outside Faction's knowledge. "Forever" is already the behavior
today if a consumer simply never constructs that command.

Where this question does apply is one layer up, in each consumer's own
config (e.g. a `deadline_ms: u64`-style field). Once section 2 lands, that
knob gets considerably lower-stakes than it looks today: firing
`DeadlineExpired` or not no longer changes whether progress continues, only
whether an observability signal gets recorded. Worth doing at the consumer
level, but it's each consumer's own config decision, not a Faction change.

---

## 6. Responsibility boundary: Faction vs. the consuming protocol

**Working principle:** Faction owns anything derivable purely from injected
config plus the history of commands already processed — pure accounting,
no judgment about what a fact means or what to do about it. The protocol
owns its own fault model (and therefore any formula derived from it), its
policy for reacting to a fact, and all I/O.

Applied to the concrete cases raised so far:

| Concern | Owner | Why |
|---|---|---|
| Quorum threshold, the number itself | Protocol | Encodes the protocol's own fault model — IBFT is Byzantine (`2N/3+1`), Raft is crash-fault-only (`N/2+1`). Faction cannot know which is correct for a given consumer. Already correctly injected via `QuorumPolicy`; the `Some(node_count)` bug in both IBFT and Raft was a protocol-side implementation mistake, not a boundary violation. |
| Counting confirmations against that threshold | Faction | Pure bookkeeping, no protocol knowledge needed. Already correct. |
| Whether a deadline exists, and how long it is | Protocol | Already correct — `Config` has no duration field (section 4). |
| What a fired `DeadlineExpired` *means*, what to do about it | Faction reports; protocol decides | Currently wrong — see sections 2–3. Turning it into a hard dead-end state is Faction imposing a policy answer ("missing this means stop") that forecloses protocols that would legitimately want "log and continue" as much as ones that want "log and stop." |
| Whether to acknowledge a rejoining known peer | Faction | Reduces to "is this a configured member" and "am I currently confirmed" — purely mechanical, zero protocol-specific judgment. A consumer *could* replicate this itself (`Bootstrapper` already has the data), but letting every consumer reimplement identical accounting is the exact failure this crate exists to prevent — see the identical `Some(node_count)` bug landing independently in both IBFT and Raft. |
| Actually sending the reply message, wire format | Protocol | Unconditional. IBFT reuses `PeerReady`; Raft's equivalent is a different type Faction has never heard of. Faction says "acknowledge-worthy: yes" as an outcome; the consumer decides how. |

One addition under this framework: `Command::DeadlineExpired` stays in
Faction's vocabulary even after losing its teeth, because Faction's own
"every transition, query, and rejection reaches the `Observer`" rule makes
it the natural single place to centrally record "this took longer than
target" alongside everything else in the lifecycle — recording centrally is
still accounting, even once it no longer triggers a policy decision.

---

## 7. Should Faction allow the quorum size to change?

Not a bare setter — that would reintroduce, through a side door, the exact
hazard Phase 3 already exists to prevent. If threshold can be swapped on one
node without coordination, two disjoint subsets of the cluster can each end
up running a different threshold, each independently convinced it holds
quorum — a safety violation, not a liveness hiccup. Phase 3's own stated
invariant ("at no point do two disjoint subsets of nodes each believe they
form a valid quorum," enforced by making single-change-at-a-time
structurally impossible) is precisely the machinery this needs, not
something to bypass by adding mutability to `Config` in Phase 0.

**Refinement:** the change should be exposed through the same mechanism as
everything else — a genuine new `Command` variant (something like
`Command::QuorumPolicyChanged { new_policy: QuorumPolicy }`), evaluated and
scoped per state through the normal `accept()`/`step()` gating, inheriting
exhaustive `(state, command)` coverage and `Observer` routing like every
other command, rather than an out-of-band setter that bypasses the state
machine's own input gating entirely. That's a necessary foundation and a
real improvement over a bare setter — but it doesn't by itself solve the
cross-node coordination problem. Being a well-formed, per-state-gated
command makes the *local* decision explicit and testable; it's still Phase
3/4's commit/abort sequencing that has to guarantee the *cross-node*
agreement about when the new value takes effect. The two are complementary,
not substitutes for each other.

Faction still never computes what the new threshold *should* be, under
either version — that stays the protocol's fault-model-driven formula,
recomputed and injected by the consumer, same as at construction.

---

## 8. Verification plan for the `DeadlineExpired`/rejoin-ack fix, once implemented

Scoped to Faction's own test suite — three tiers, from state-machine unit
level up to real multi-process convergence. Depends on sections 2–4 above
actually being decided first, since the exact shape of what's being tested
(does `TimedOut` still exist as a state?) isn't settled yet.

- **`protocol` (`protocol/tests/protocol_tests.rs`)**: add
  `decide_deadline_expired_before_quorum_does_not_produce_dead_end_state` —
  drive `Protocol::decide()` through a `DeadlineExpired`-translating input
  before quorum, assert the resulting state still accepts a subsequent
  `ParticipationObserved` rather than producing `Noop` forever after.
- **`protocol-validation` (`protocol-validation/tests/`)**, using the
  existing deterministic `Cluster` harness:
  - `deadline_expired_tests.rs` — its two existing tests,
    `deadline_expired_exits_with_timed_out_and_cancels_pending_timers` and
    `all_nodes_time_out_when_deadline_fires_before_quorum`, currently assert
    `is_timed_out(i)` plus no pending timer work as the *correct* outcome.
    That is exactly the behavior being removed, so both need their
    assertions rewritten — not just new siblings added alongside unchanged
    old ones — to confirm the node stays receptive and can still reach
    `Bootstrapped` later. New in the same file:
    `deadline_expired_then_late_participation_observed_still_reaches_bootstrapped`.
  - `late_arrival_tests.rs` — add
    `node_rejoins_after_own_deadline_and_after_peers_already_bootstrapped_still_converges`:
    peers reach `Bootstrapped` without node 0, node 0 times out locally,
    a late ack arrives, node 0 still converges.
  - `lost_ping_tests.rs` / `dropped_ready_tests.rs` — add the staggered,
    non-overlapping two-node case: node A's ping/ready is dropped and later
    resent while node B is independently mid-cycle; both converge
    independently.
  - New file `quorum_boundary_tests.rs` — the negative/regression guard:
    construct a cluster where simultaneously-unreachable members exceed
    N − quorum, assert it does *not* reach `Bootstrapped` and does nothing
    unsafe. This is the one that proves the fix didn't quietly erode the
    quorum floor while removing the dead end.
  - Harness gap to close first: `Cluster` has `start_node`/`start_all` but
    no way to simulate a member going away and coming back — needs a
    `restart_node`, or confirmation that `start_node` is safe to re-invoke
    on an already-started index.
- **`system-tests` (`system-tests/tests/convergence_tests.rs`)**: the
  existing spawn × transport × timer-delay × quorum matrix only exercises
  the clean-boot path. Add a sibling parametrized test that kills one real
  process/task/thread mid-bootstrap and restarts it, asserting
  `is_bootstrapped()` still eventually goes true. Real-process tier is
  expensive — scope to a representative subset of the matrix rather than
  the full case list unless full coverage is wanted there too.

---

## 9. Decision (2026-07-19): §1–§4 are a Phase 0 bug, fixed before Phase 1

Confirmed direction. The rejoin-ack limitation (§1), the `TimedOut`
dead-end (§2), the `DeadlineMissed` non-terminal outcome (§3), and the
removal of `TimedOut` as a distinct internal state (§4) are treated as a
**bug in Phase 0** — fixed in place in the existing Phase 0 state machine,
before any Phase 1 work begins. This is **not** a new roadmap phase; the
ROADMAP is unchanged. The fix stays entirely within static membership; it
changes only how an already-known set re-converges after a member's
transient absence, and removes a dead-end that locks a node out after a
premature deadline.

Two things forced the decision:

1. **A second independent consumer hit it** — see the 2026-07-19 note in §1.
   Two consumers reinventing the same reconnect accounting is the precise
   failure Faction exists to prevent.
2. **The thin-adapter target makes the §6 boundary testable.** If fixing
   this in Faction lets the downstream workaround collapse to a forwarding
   shim, that *is* the proof the boundary was drawn in the right place.

**Fix surface (draft, pending its own spec):**

- `TimedOut` stops being a dead end — `DeadlineExpired` before quorum keeps
  the node receptive; it can still reach `Bootstrapped` later. (§2, §4)
- `DeadlineExpired` becomes non-terminal, repeatable, accounting-only — new
  outcome `DeadlineMissed { confirmed_count }` replaces
  `Concluded { TimedOut }` in `Pinging`/`Collecting`; `Bootstrapped` and
  `Initial` still reject it as a stale/early timer. (§3)
- Rejoin-ack outcome — a `ParticipationObserved` from a configured member,
  arriving at a node already past its own local completion, yields a new
  outcome `AcknowledgeRejoin { peer_id }`: "reply, this is a known member
  reconnecting." The acknowledge-worthy *decision* is Faction's; the wire
  reply stays the consumer's. (§1 under the §6 boundary)

**Thin-adapter target** — what etheram-ibft's `Bootstrapper` workaround
becomes once the fix ships. This table *is* the acceptance criterion:

| Consumer carries today (workaround `8fba86c`) | After the fix |
|---|---|
| `locally_completed: bool` tracking "done locally, awaiting quorum" | gone — Faction knows this from its own state |
| `handle_peer_ready` branch: ack rejoin ping while locally-complete | map `AcknowledgeRejoin { peer_id }` → send wire reply |
| `handle_retry`: rebroadcast readiness while awaiting quorum | map re-emitted `BroadcastLocalReady` (on the consumer's retry tick) → rebroadcast |
| `is_readiness_adaptation_message` routing-gate widening | unchanged — consumer wire concern, stays local |

etheram-raft, which has none of this logic yet, wires the same two outcomes
from the start and never grows the workaround.

**Test re-baselining (paper-relevant).** Because this is a bug fix to
Phase 0 rather than a new phase, §8's rewrite of the two existing
`deadline_expired_tests.rs` assertions is simply what fixing the bug means —
they currently assert the dead-end behavior, so they are corrected to
assert the fixed behavior. Phase 0's final, canonical form is the corrected
one; the strict-superset property ("Phase N tests pass unchanged at
Phase N+6") holds from that corrected baseline, because a bug fix
re-defines what Phase 0 *is* rather than layering a new phase on top of a
known-wrong one. Worth a one-line note in the paper's methodology so a
reviewer who sees a rewritten Phase 0 test reads it as a correction, not a
loosening.

---

## 10. Decision (2026-07-19): Faction is persistency-free (named invariant)

**Faction persists nothing, ever.** This is now a named, first-class
invariant — not merely a consequence of the existing "the machine never
performs I/O" rule, but the stronger statement from which that rule *and*
the Phase 3–5 membership source-of-truth boundary both follow. It resolves
the source-of-truth question left open in earlier drafts of this doc.

**Persistency-free ≠ stateless.** Faction is emphatically stateful, and its
state is *expected to grow significantly* as the machine surfs Phases 1→6
and beyond — member set, in-flight-change guard, epoch counter, and
whatever later phases add. What Faction never does is *durably own* any of
that across a restart: state lives in memory and is reconstructed by
deterministic replay, never restored from a Faction-owned store. Anyone
tempted to "simplify" Faction toward statelessness is breaking the Mealy
model; anyone tempted to let it *remember* the member set across a reboot is
breaking this invariant. Both temptations arrive precisely at Phase 3–5.

**Two theorems fall out for free:**

- *Log-authoritative membership.* If Faction persists nothing, it cannot be
  a competing durable source of truth. The consumer's committed log (Raft
  log entries; IBFT validator-set updates applied at a height) is the sole
  authority, by construction — inheriting Raft's adopt-on-append /
  roll-back-on-truncation semantics for free instead of re-implementing
  them in a parallel store that could diverge.
- *No I/O.* Persistence is the only I/O a pure state machine would be
  tempted into; forbidding it structurally upholds the existing "never
  performs I/O" rule rather than relying on discipline to keep it.

**Consumer contract.** Deterministic, ordered replay of committed
config-change inputs is the consumer's durability obligation. **Faction owns
the fold; the consumer owns the disk.** Identical across Raft and IBFT.

**The one cost, named honestly.** On reboot the consumer must replay
membership history through Faction to rebuild the in-memory state —
unbounded for a long-lived cluster with many changes. The mitigation keeps
Faction pure: the *consumer* snapshots the derived view ("at log index X,
members = {…}, epoch = N") and replays only the tail past the snapshot. Not
a workaround — it is exactly how Raft already compacts (a Raft snapshot
includes the committed configuration as of the snapshot), and it reuses the
very snapshot mechanism `etheram-raft/docs/RAFT-SPEC-COVERAGE.md` lists as a
gap, now doing double duty: joiner catch-up *and* the membership-replay
bound.

**Consequence for the IBFT integration — "govern, don't replace."** Because
the log stays authoritative, Faction does not replace IBFT's existing
validator-set-update mechanism; it sits in front of it as the guard/intent
layer (is this change safe to schedule, is one already in flight) while
IBFT's scheduled-at-height application stays the execution layer. For Raft,
which has no membership mechanism yet, Faction is the first — same boundary,
greenfield. Worth an explicit confirmation before Phase 3, but it follows
directly from persistency-free.

---

## 11. Bug (RESOLVED 2026-07-19): `ProcessResult::Accepted` omitted `admissible`

`Rejected` and `Probed` both carried `admissible: Vec<Command>`; `Accepted`
carried only `outcomes` and `cluster_view`, forcing a consumer to issue a
separate `Probe` to learn what was valid after an accepted transition.

**Resolved:** `Accepted` now carries `admissible` too — computed from the
state the transition produced, identical to what a follow-up `Probe` returns.
All three `ProcessResult` variants surface it consistently. Covered by the
valid-transition matrix (admissible asserted on every accept-path case) and by
entry-point tests, including one that asserts `Accepted`'s admissible equals a
subsequent `Probe`'s. See `docs/ADRs/P1-ADR-SingleEntryPoint.md`.

---

## 12. TODO: Tier-2 ADRs to write

Real decisions, ranked below the P0/P1 core in `docs/ADRs/`. To be written
later:

- [ ] `P2-ADR-TotalObservability` — every transition, query, and rejection
  reaches the `Observer`, as a channel distinct from `Outcome`s.
- [ ] `P2-ADR-StateAsTraitObject` — one struct per state, `Box<dyn State>`,
  keeping Phase 1→6 growth additive.
- [ ] *(blocked on §7)* `Config` immutability — membership/quorum changes
  arrive as `Command`s, never as setters.
- [ ] *(deferred to post-Phase-6)* `PeerId` genericization (currently `u64`).
