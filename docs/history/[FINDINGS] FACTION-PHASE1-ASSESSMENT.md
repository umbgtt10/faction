# Faction Phase 1 — Assessment & Design Feedback

Companion to [`faction-phase1-brief.md`](./faction-phase1-brief.md). Part 1 is
the read-only assessment requested before any design decision is committed —
every claim below is grounded in a direct read of the current `core/src` tree
(all 25 source files) and the existing `core/tests/transition_matrix/` suite,
not inferred from the brief. Part 2 states the open design questions with
alternatives, a cost-benefit comparison, and a recommendation for each. No
code was changed to produce this document.

---

## Part 1 — Assessment

### 1.1 Type-level invariant audit (bears on brief §6)

**Finding: no genuinely compile-time-only invariant was found anywhere in
`core/src`.** Every state (`Initial`, `Pinging`, `Collecting`, `Bootstrapped`)
implements `State` as a plain trait object (`Box<dyn State>`); nothing about
the type system restricts which `Command` variant can be offered to which
state's `step()`. What looks, from the outside, like "this transition cannot
happen" is in every case a **two-tier runtime contract**, not a structural
wall:

1. `Faction::process()` (`core/src/faction.rs:42-83`) intercepts
   `Command::Probe` first (returns `ProcessResult::Probed` without calling
   `step()`), then checks `self.state.accept(&command)` — if `false`, returns
   `ProcessResult::Rejected` without calling `step()` either.
2. Only a command that passes both gates ever reaches `step()`. Every
   `unreachable!()` inside a `step()` implementation
   (`states/pinging.rs:175` for `Probe`; `states/collecting.rs:149` for
   `LocalParticipationCompleted`; `states/bootstrapped.rs:67` for anything
   `accept()` rejects) is provably dead code **given `process()`'s own
   calling order** — it is an internal consistency assertion protecting
   against a future refactor breaking that order, not a case Rust's type
   system makes impossible to construct. A Go port replicates this by
   implementing the same two-gate order in its own dispatcher; nothing here
   requires a type system Go lacks.

**Consequence for §6:** the "for each invariant, state whether it's enforced
statically or dynamically" exercise the brief asks for may have an
**empty static column** for this reference implementation. That should be
stated as a checked, falsifiable claim in `SPECIFICATION.md` §9 — "no
statically-enforced invariant was identified; every invariant is enforced by
the `Probe → accept() → step()` order in `Faction::process()`" — rather than
assumed silently. See design question 2.3.

### 1.2 `ProcessResult` and `Outcome` are two independent response tiers

The brief's illustrative effect vocabulary (§3.1) reads as if there is one
flat "effect" concept. There are actually two, nested:

- **`ProcessResult`** (`core/src/process_result.rs`) — always returned by
  `process()`: `Accepted { cluster_view, admissible, outcomes }`,
  `Rejected { cluster_view, admissible }`, `Probed { cluster_view,
  admissible }`. `Rejected` carries no reason beyond "not in `admissible`" —
  it is the surface form of the §1.1 runtime gate.
- **`Outcome`** (`core/src/outcome.rs`) — the `Vec<Outcome>` nested inside
  `Accepted`, the result of a command the state *did* step. Several
  `Outcome` variants are themselves domain-level "soft rejections"
  (`JoinDenied`, `NonMemberIgnored`, `DuplicateMemberIgnored`,
  `DuplicateParticipationIgnored`, `DuplicateReadyIgnored`) distinct from
  real effects (`MemberAdmitted`, `EmitJoinRequest`, `Concluded`, ...).

Both tiers need canonical representation; a canonical model built only from
`Outcome` (matching the brief's example shape) would miss the
`ProcessResult::Rejected` tier entirely. See design question 2.1.

### 1.3 Reachability of the existing test setup (`Init` enum)

`core/tests/transition_matrix/builder.rs`'s `Init` enum
(`Fresh`, `PingingPeer1Confirmed`, `CollectingAlmostQuorum`, `Bootstrapped`,
etc.) is **already reachable-by-construction**: `build()` always starts from
a fresh `Faction::new(...)` and drives it there purely by calling
`faction.process(Command::...)` in sequence — there is no direct-state-
injection shortcut anywhere in the harness. This directly satisfies brief §5;
converting an `Init` case into a vector's `"setup": [...]` array is close to
mechanical.

One quirk worth carrying into the exporter: `build()` unconditionally issues
a `ParticipationObserved { peer_id: 99 }` first (peer 99 is not a member) for
every `Init` variant except `Initial`, purely to force the FSM out of the
`Initial` struct into a real `Pinging` instance — the command itself is a
no-op (`NonMemberIgnored`). This is a harness convenience baked into
`builder.rs`, not part of `Command`'s real vocabulary; the exporter should
either reproduce it verbatim in `setup` or confirm it can be dropped (worth
one direct check, not assumed either way).

### 1.4 Public API surface

`core/src/lib.rs` exposes 15 `pub mod`s; the only way to drive the machine is
`Faction::new()` + `Faction::process()`, both ordinary public methods — no
`pub(crate)`, `#[cfg(test)]`, or test-only constructor was found anywhere in
`core/src`. A `harness/rust/` built against the public API only (brief §10)
has nothing to work around today.

### 1.5 "Next step: system tests" — already closed

The brief's premise (workstream sequencing) assumed this might still be open.
It is not: `feat/phase-1-joining`'s own history shows
`efa36bf docs: mark Phase 1 (dynamic joining) complete`, preceded by
substantial, already-landed `system-tests` work (per-run log folders,
process-spawn helpers, `FaultyTransport` fault injection, parallelization via
`slotgate`, join scenarios broadened to Thread and Process spawn). There is
no pending system-tests step to fold into the vector work — see design
question 2.4 for what this means for sequencing.

### 1.6 Current shape of "the matrix" (bears on §7's refactor scope)

"The matrix" is not one file today; it is **six**, in two different idioms:

- `state_transition_matrix_tests.rs` — `rstest` `#[case::name(...)]` macros
  for the *accepted* path (command steps, expected `Outcome`s, expected
  `ClusterView` assertions).
- `{initial,pinging,collecting,bootstrapped}_invalid_tests.rs` — one file per
  state, same `#[case]` idiom, for the *rejected* path (`ProcessResult::Rejected`,
  asserting the `cluster_view` is unchanged and `admissible` matches
  expectation).
- `admissible_invariant_tests.rs` — a property test cross-checking that the
  `admissible` list `Probe` reports is exactly consistent with what
  `accept()`/`Rejected` actually does for every command.

None of this is freestanding data today — every case lives inside an
`rstest` macro invocation. Brief §7's "the main refactor of this phase" is
real and is exactly this: extracting all six files' cases into one
`matrix.rs` table with rows for both outcomes (`Accepted{outcomes,...}` /
`Rejected{admissible}`), consumed by the existing tests and by a new
exporter. The property test (`admissible_invariant_tests.rs`) is a
cross-check *over* the matrix, not matrix data itself, and should stay a
separate, ordinary test.

### 1.7 Repo-wide check: `property_tests/`, `protocol`, `protocol-validation`, `core-validation`

The audit in 1.1 was scoped to `core/src`. Extending it across the rest of
the repository changes nothing about the "no static invariant found"
conclusion, and adds three findings that materially help feasibility:

- **`core/tests/property_tests/{invariant_tests,model_tests,safety_tests}.rs`**
  are all `proptest`-driven behavioral checks (counts never decrease, exit is
  terminal and idempotent, non-member/duplicate commands never mutate state,
  quorum-exit implies pinging-completed) — confirming, not contradicting,
  1.1: every property here is checked over generated runtime input, none of
  it is a type-level constraint.
- **`model_tests.rs` contains an independent, from-scratch `ModelCoordinator`**
  that duplicates the state machine's behavior in a second, simplified
  implementation with no access to `core`'s internals, proptest-checked for
  exact equivalence with `Faction` (both outputs and `ClusterView`) across
  randomly generated command sequences up to 64 long. This is materially
  relevant: it is already-existing, already-passing evidence that the domain
  logic can be correctly expressed by a second, independent implementation —
  which is most of the real intellectual risk in "can this be specified
  language-agnostically" retired before Phase 1 even starts.
- **`protocol`/`protocol-validation` are a cleanly separated consumer-adapter
  layer, not a wire-format leak into `core`.** `protocol::Protocol` wraps a
  `Faction` and adds `InputMessage`/`OutputMessage`/`TimerMessage`/
  `MessageTranslator` — a concrete illustration of "you bring the network,
  the transport" (the crate's own doc-comment), kept in its own crate
  specifically so it can be wire-format-opinionated without `core` being so.
  Invariant #2 ("no wire format... in `faction`") holds for `core`, the
  actual specification target; `protocol` existing alongside it is additive,
  not a violation.
- **`core-validation` (`cluster_simulation.rs`, `scenario_harness.rs`,
  `scenario_node.rs`, `scenario_event.rs`, `scenario_trace_entry.rs`) is a
  multi-node simulation harness** — a near-direct source for brief §5.2's
  scenario vectors (multi-node join/bootstrap sequences), not something to
  build from nothing.

---

## Part 2 — Design questions

### 2.1 How should the two rejection tiers (§1.2) map to canonical effects?

**A — Map each existing variant to its own named canonical effect, as-is.**
`ProcessResult::Rejected` becomes one effect (e.g. `rejected_not_admissible`,
carrying `admissible`); each of the five `Outcome`-level soft-rejections
becomes its own effect (`join_denied`, `non_member_ignored`,
`duplicate_member_ignored`, `duplicate_participation_ignored`,
`duplicate_ready_ignored`). No Rust restructuring — purely an additive
mapping-layer exercise.
*Cost:* a wider effect vocabulary (9-10 variants instead of one generic
`rejected + reason` pair). *Benefit:* every effect is already backed by a
real, tested Rust variant — zero invented taxonomy, zero risk of the mapping
layer guessing at categories the code doesn't actually distinguish.

**B — Unify all rejections into one `{"effect": "rejected", "reason": <enum>}` shape.**
Requires inventing a `RejectReason` enum in the canonical model spanning both
tiers, and deciding whether `admissible` becomes a universal field on every
rejection effect (today it's only on `Rejected`/`Probed`, never on the
`Outcome`-level soft-rejections).
*Cost:* real design work to build a taxonomy the Rust code doesn't currently
express as one flat type, with a genuine risk of inventing distinctions (or
collapsing real ones) that don't match actual behavior. *Benefit:* matches
the brief's illustrative JSON shape (§3.1) more literally, and gives future
rejection kinds one place to be added.

**Recommendation: A.** The brief's own requirement is that rejection reasons
be *enumerated*, not that they share one field shape — a closed set of
9-10 distinctly-named, already-verified effects satisfies that. §3.1 also
says explicitly to "substitute real effects" for its illustrative names; I'd
take that literally rather than restructure working code to resemble the
illustration.

### 2.2 Canonical ordering for collections (`Vec<PeerId>` etc.)

**A — Keep insertion order, document it as canonical.** `step()`'s purity
already makes insertion order fully deterministic and reproducible from a
given command sequence; nothing needs to change in `core/src`.

**B — Sort by identifier before serializing.** Matches the brief's own
suggestion ("sorted by identifier is fine") and may read more naturally in a
hand-inspected vector file.

*Cost-benefit:* B adds a sort step in the mapping layer for a property (order)
that's already deterministic and already comparable by deep equality either
way — it buys marginal readability, not correctness. A costs nothing.

**Recommendation: A.** Document insertion order as canonical rather than add
a transformation whose only benefit is cosmetic.

### 2.3 What counts as evidence for an `unconstructible_in` claim?

Given §1.1's finding — no static-only invariant identified across the full
`core/src` tree — this question is lower-stakes than it looked before the
audit, but not zero-stakes: a future refactor (e.g. a typestate rewrite)
could introduce a real one, and the brief is right that an unverified
`unconstructible_in` tag is a silent-coverage-loss trapdoor.

**A — No mechanism now; revisit if a real case appears.** Costs nothing today
given the audit found zero candidates. Risk: if the audit missed something,
nothing catches it later either.

**B — Require a comment citing the exact enforcing mechanism wherever
`unconstructible_in` is used, checked in review.** Lightweight, costs one
sentence per (currently: zero) use.

**Recommendation: B, but scoped to "when first needed."** State the §1.1
finding as an explicit, falsifiable line in `SPECIFICATION.md` §9 ("no
statically-enforced invariant identified; verified against the full
`core/src` tree on `<date>`") so it's a claim a future contributor can
re-check rather than an assumption — and require the evidence-comment rule
the moment (if ever) a real `unconstructible_in` case is proposed, rather
than building the mechanism speculatively for a set that's currently empty.

### 2.4 Sequence the spec work against `feat/phase-1-joining`, or wait for `main`?

**A — Build the canonical model against `feat/phase-1-joining` now.** The
join vocabulary (`JoinRequested`/`JoinApproved`/`JoinRejected`/
`EmitJoinRequest`/`MemberAdmitted`/`JoinDenied`/`DuplicateMemberIgnored`/
`AcknowledgeRejoin`) is already load-bearing in `Command`/`Outcome`, and
§1.5 confirms Phase 1 (dynamic joining) is already marked complete with its
own system-tests landed — there is no smaller, "pre-joining" baseline left
to design against.

**B — Merge to `main` first, start spec work on a clean baseline.** The only
actual benefit is git-history tidiness (spec-ification as its own,
cleanly-separated commit sequence); the code content is identical either
way once merged.

**Recommendation: A.** Merge `feat/phase-1-joining` to `main` as its own,
unrelated step whenever convenient, but don't gate the start of
spec-ification work on that merge landing first — waiting buys organizational
neatness, not technical safety, and the join vocabulary is already the
correct target either way.
