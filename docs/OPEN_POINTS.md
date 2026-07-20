# Open Points

Status: **Phase 1 (dynamic joining) — core join axis + join-capable harness,
gate-green.** The ClusterView builder/DTO split landed, so live membership is now
observable. The join system tests run on Task × In-Memory: scenarios 1
(join-then-converge), 3 (rejected), 4 (duplicate), and 6 (concurrent multi-join)
are green. Remaining: scenario 2 (blocked on decision #6 below — `Collecting`
rejects a member's participation), scenario 7 (needs deadline-injection harness
support), and broadening to the Thread spawn and the other transports; scenario 5
stays postponed (see the join subsection). The section below captures the design,
the decisions now locked, and the longer-standing open questions raised while
building Faction.

Companion to `ROADMAP.md`. Items move into `ROADMAP.md` (or a phase's own spec)
once decided; until then they stay here rather than clutter the roadmap with
unsettled debate. Settled architectural decisions live in `docs/ADRs/`; shipped
changes live in `CHANGELOG.md`.

---

## Phase 1 — dynamic joining: design considerations

Phase 1 answers *"can a new peer join the cluster?"* The peer set is no longer
fixed at construction: a peer can be admitted at runtime, after which its signals
count as member signals. Draft I/O (from `ROADMAP.md`, may change):
`JoinRequested` / `JoinApproved` / `JoinRejected` in; `EmitJoinRequest` /
`MemberAdmitted` / `JoinDenied` out.

### The membership axis is orthogonal to the bootstrapping progression

The three join commands form a **membership axis** independent of the
`Initial → Pinging → Collecting → Bootstrapped` progression. In every state the
behaviour is identical, and none of them advance the progression — they stay in
the current state. That is *why* the strict-superset rule holds almost for free:
the existing progression transitions are literally untouched.

| Command | Behaviour in every state | Emits | Membership |
|---|---|---|---|
| `JoinRequested { p }` | accept, stay | `EmitJoinRequest { p }` | none — forwards to the caller for a decision |
| `JoinApproved { p }` | accept, stay | `MemberAdmitted { p }` (or `DuplicateMemberIgnored { p }` if already a member) | **adds `p` to the live member set** |
| `JoinRejected { p }` | accept, stay | `JoinDenied { p }` | none |

This encodes the ROADMAP invariant *"faction never decides admission policy"*: on
a request it only forwards; it changes membership solely when the caller says
`JoinApproved`. The payoff test: a peer whose `ParticipationObserved` was
`NonMemberIgnored` before admission becomes `ParticipationAccepted` after.

Proposed edge handling (policy-free, open to change): forward `JoinRequested`
even for an existing member; a re-approve is `DuplicateMemberIgnored` (reusing the
existing `Duplicate*Ignored` idiom).

### The genesis peer set is the right primitive

Faction is — and should stay — constructed with a pre-packaged genesis peer set.
This is not a limitation; it is the universal integration contract (every current
consumer already does it: IBFT/Raft build
`Config::new(local, validators, QuorumPolicy::new(threshold))`). Genesis
membership is near-universal — a Raft initial config, a consensus genesis
validator set, etcd's `initial-cluster`. Dynamic membership *layers on top of* a
seed; it never replaces it. A start-from-empty design has no quorum baseline and a
chicken-and-egg admission bootstrap. Phase 1's job is to reframe the genesis set
as **seed, not ceiling**: `Config` = immutable genesis, live `Members` =
genesis ∪ admitted. The only trap is conflating "genesis" with "forever-fixed" —
the exact limitation (L1) Phase 1 removes.

### Member-set representation (decided: carried in each `State`)

Membership is immutable today — the three `config.is_member()` call-sites read
`Config.peers`. Phase 1 makes it mutable, which raises where the live set lives.
**Decision: carry it in the `State` objects (representation A), via a `Members`
value object seeded from `Config.peers` (genesis) and grown on admission.**

Why A, not machine-owned (B):

- **Membership is authoritative, mutable state** — not immutable genesis (that is
  `Config`) and not a derived read-model (that is `cluster_view`, recomputed each
  step). Authoritative mutable state is exactly what the `State` trait objects are
  for; it belongs with them.
- **`process` stays pure.** Under A, `step` returns a new state that already
  carries the new membership — no wrapper bookkeeping. Under B, `step` cannot
  mutate a wrapper field, so membership growth would have to be returned as an
  extra value or *inferred* by `process` from the `MemberAdmitted` outcome,
  coupling the machine loop to outcome semantics.
- **The roadmap is membership-dominated.** Phases 2–6 are all membership work
  (liveness of members, add, remove, epochs/rejoin, concurrent changes). Putting
  membership where the transitions happen keeps growth additive per
  `P2-ADR-StateAsTraitObject`; putting it in the wrapper would accrete a
  membership god-object there while the states go anemic.
- **Self-containment.** A `State` should fully describe its phase; splitting the
  state vector across the boxed state and wrapper fields leaks that description.

Cost (accepted): every state carries `Members` — including `Initial`, which stops
being a unit struct (it holds the genesis set and passes it to `Pinging`) — and
the constructor changes ripple mechanically into tests, the same kind of change
Phase 2 already made when `Collecting::new` gained a parameter. A `Members` value
object (own file: `is_member`, `with_admitted`, `len`) keeps the threading clean.

`Config.peers` stays as the immutable **genesis** seed (used to construct the
initial `Members` and to replay/reset); the live set evolves in state. This is now
locked in **`P1-ADR-ConfigIsImmutableGenesis`**: `Config` = genesis, the
state-carried `Members` = live, mutated only by an admission `Command`.

### Collecting must stay receptive to a member's participation (from the IBFT integration)

The terminal-sink fix made `Bootstrapped` re-advertise (`AcknowledgeRejoin`) to a
member that (re-)sends `ParticipationObserved`. `Collecting` still **rejects**
`ParticipationObserved` outright — a Phase-0 assumption ("my participation phase is
over"), which is *phase*-based, not *membership*-based. Phase 1 reopens that cell:
once a peer is an admitted member, "its subsequent signals are treated as valid
member signals" — including participation that arrives while the local node is in
`Collecting` (locally completed, awaiting readiness quorum).

**Decision for Phase 1:** a member's `ParticipationObserved` in `Collecting` should
be *acknowledged, not dropped* — either re-advertise like `Bootstrapped`
(`AcknowledgeRejoin`, stay) or actually accept/track it. Pick one deliberately and
give it matrix + admissible + property coverage. This generalizes "no silent sinks"
from the terminal state to the whole post-local-completion lifecycle.

Surfaced by the IBFT integration: IBFT's rejoin workaround is *broader* than
Faction's current `Bootstrapped`-only `AcknowledgeRejoin` (it re-advertises from
local-completion onward, i.e. across `Collecting`), so IBFT keeps that workaround
until Faction closes this gap — at which point it collapses cleanly. See
`etheram-ibft/docs/FACTION-PHASE-1-INTEGRATION.md`.

### Increment ladder (TDD, gate-green at each step)

1. Extend `Command` + `Outcome` with the 3 inputs / 3 outputs.
2. `Members` value object carried in each `State`, seeded from `Config.peers`;
   move the three `is_member` sites onto the carried set (behaviour-identical
   while the set is genesis-only, so Phase 0 stays green).
3. `JoinRequested → EmitJoinRequest`, all four states (shared `JoinStep`, matching
   the existing `*Step` pattern) + matrix + admissible cells.
4. `JoinRejected → JoinDenied`, same.
5. `JoinApproved → MemberAdmitted` / `DuplicateMemberIgnored`, growing the
   carried member set.
6. Payoff: post-admission member signals count (`NonMemberIgnored → Accepted`).
7. Exhaustive-matrix + admissible-invariant + property-model updates; the
   `Config`-immutability ADR; docs; `run_stage_1` + `run_stage_2`.

**Status: increments 1–7 are implemented and green** through `run_stage_1` +
`run_stage_2` — the 32-case `(state, command)` matrix, the admissible-invariant,
the property-model, and `core/tests/join_tests.rs` all pass; the ADR is written
(`docs/ADRs/P1-ADR-ConfigIsImmutableGenesis.md`). What remains for Phase 1 is the
join-capable harness + `system-tests/tests/joining_tests.rs` (the subsection below)
and decision #6.

### System tests: the harness needs a join capability

The current `system-tests` harness is **fixed-size and one-shot** and does **not**
support adding a node: `ClusterBuilder::new(node_count, node_required)` builds a
static `0..node_count`, transports are built as a complete **mesh up front**
(`new_mesh(&peer_ids)`), and `Cluster` exposes only `start_all` / `step_all` /
`poll_until_bootstrapped` — there is no `add_node`. Phase 1 therefore needs a
harness workstream, roughly co-equal with the core change:

1. `Cluster::join(peer_id)` (+ a process-spawn analog).
2. A **"connect a late peer into a live mesh"** op per transport. **In-memory
   only for the first pass** — an in-process link insertion, the simplest case.
   The harder transports (Channels pairwise links; TCP/gRPC late socket connect)
   and the Process-spawn join are deferred to the broadening phase below.
3. A **join driver**: the newcomer emits its signal, the harness routes
   `JoinRequested` to members, the test's approver policy answers
   `JoinApproved`/`JoinRejected`, and the protocol maps outcomes to the wire.

**Scenarios to lock in advance** — a new file `system-tests/tests/joining_tests.rs`
(leaving `convergence_tests.rs` byte-untouched — the cleanest proof of the
strict-superset rule; it matches the one-file-per-concern pattern in
`protocol-validation/tests/`):

| # | Scenario | Proves |
|---|---|---|
| 1 | join-then-converge (cold newcomer joins a bootstrapped cluster) | admission + later signals now count |
| 2 | join-before-bootstrap (admitted mid-Pinging/Collecting) | membership can grow in-flight |
| 3 | join-rejected | `JoinDenied`; signals stay `NonMemberIgnored`; cluster unaffected |
| 4 | duplicate-join | `DuplicateMemberIgnored`; count stable |
| 5 | join-raises-quorum (admit + caller injects larger threshold) | membership/quorum decoupling |
| 6 | concurrent multi-join | Phase-1 permissive/stateless semantics (vs Phase 3's coordinated add) |
| 7 | join-after-deadline-miss | ties Phase 1 to the terminal-not-sink fix (0.4.0) |

**Scenario 5 (join-raises-quorum) is postponed — not built in this pass.** It
presupposes the caller can inject a larger threshold, which needs a
`QuorumPolicyChanged` command that does not exist yet: an uncoordinated threshold
swap is exactly the safety hazard the quorum-change question below defers to
Phase 3. The one thing Phase 1 *can* assert — that admission alone never moves
the threshold as the set grows (decision #3) — is already covered at the core
level (`join_tests.rs::admission_does_not_change_the_quorum_threshold`), so
re-proving it as a system test buys little. The minimal-first join matrix is
therefore scenarios **1–4, 6, 7**; scenario 5 returns once the quorum-change
command lands.

**Matrix, minimal-first (decided):** run the scenarios on **Task/Thread ×
In-Memory only** to begin with — the simplest spawn models and transport. Broaden
to the other transports (Channels, TCP, gRPC) and the Process spawn **only once
the core join axis and this minimal matrix are fully green**. This keeps the
hardest harness work (late socket connect, process-spawn join) off the critical
path until the feature is proven on the simplest substrate.

### Implementation plan (the three-layer join surface)

The section title understates the reach: join is not plumbed above `core` yet, so
the "harness capability" is a **three-layer** change — protocol wire → harness →
tests — matching the "co-equal with the core change" note above. The join
*semantics* are all in `core` and green (increments 1–7); the new work is
plumbing, routing, and tests, with **no new state-machine logic**.

**Layer 1 — protocol/wire (`protocol/`), the smallest faithful surface:**

- `TransportMessage::JoinRequest { from }` — a newcomer announcing itself;
  `MessageTranslator::to_command` maps it to `Command::JoinRequested { peer_id: from }`.
- `OutputMessage::EmitJoinRequest { peer_id }` — surfaces a member's forward so the
  harness/approver can act on it (today `to_output_messages` collapses everything
  to `Noop`); `to_output_messages` maps `Outcome::EmitJoinRequest` onto it.
- `Protocol::admit(peer_id)` / `Protocol::deny(peer_id)` — the **caller** admission
  path, wrapping `Command::JoinApproved` / `JoinRejected`. Admission is a local
  caller decision (`P0-ADR-ProtocolAgnostic`: "faction never decides admission
  policy"), so it is a method, **not** a peer wire message. `MemberAdmitted` /
  `DuplicateMemberIgnored` / `JoinDenied` need no wire surface — the scenarios
  assert them behaviorally.

Keep these match arms trivial so `faction-protocol` stays under the stage-2 CRAP gate.

**Layer 2 — harness (`system-tests/`):**

- `InMemoryTransport::connect(peer_id, inbox)` + an inbox-handle accessor, so a
  late peer can be spliced into a live mesh (today `new_mesh` wires everything up
  front and there is no late-link insertion).
- `Cluster::join(peer_id)` — builds a newcomer node and connects it into the mesh.
  Newcomer genesis = **existing members ∪ self**, so it can ping, collect readiness,
  and reach `Bootstrapped` from its own side while the existing members grow to
  include it via admission (the Phase-1 asymmetry: genesis = seed, not ceiling).
- An **approver policy** the test supplies (accept-all / reject / …).
- A **join driver** running the loop: the newcomer's join signal reaches members as
  `JoinRequest` on the wire → members emit `EmitJoinRequest` → approver decides →
  harness calls `admit`/`deny` on each member → the newcomer's later pings now count.
- **Task first** (direct method calls on the node's `Protocol`), **then Thread**
  (needs a small injectable command queue the run-loop polls — Thread has no
  external admission hook today).

**Layer 3 — tests (`system-tests/tests/joining_tests.rs`):**

- New file; `convergence_tests.rs` left byte-untouched (the cleanest proof of the
  strict-superset rule).
- Scenarios **1–4, 6, 7** (5 postponed, above), Task/Thread × In-Memory.
- **Assertions read membership directly** — the `ClusterView` builder/DTO split
  (`P2-ADR-ClusterViewBuilderAndDto`) exposes live `members` on the view, so the join
  scenarios assert on `member_count` (e.g. duplicate-join keeps it stable) alongside
  the behavioral signal that an admitted peer's ping now counts and the cluster
  bootstraps with it.

**TDD increment order (gate-green at each step via `run_stage_1`):**

1. Failing Task × In-Memory **scenario 1** (join-then-converge) — the payoff:
   admission flips `NonMemberIgnored` → `ParticipationAccepted`.
2. Minimal Layer-1 + Layer-2 plumbing to green it.
3. Scenarios 2, 3, 4, 6, 7 on Task × In-Memory, one at a time.
4. Generalize the spawn axis to **Thread** (the injectable-command queue); rerun
   the matrix.
5. `run_stage_1` + `run_stage_2` green. Broadening to the other transports
   (Channels, TCP, gRPC) and the Process spawn stays deferred per the minimal-first
   decision.

**As shipped:** join is routed entirely through control-plane methods —
`Protocol::request_join` / `admit` / `deny` — with **no new wire message** (the
Layer-1 `TransportMessage::JoinRequest` / `OutputMessage::EmitJoinRequest` were not
added; they would have forced every transport's encoder to grow a variant for an
In-Memory-only pass). Newcomer genesis = existing members ∪ self; one scenario per
increment; scenario 5 postponed (above). The `ClusterView` split shipped with an
observable `members`, so assertions read `member_count` rather than inferring
membership.

### Release & consumer sequencing (decided: local path reference)

Phase 1 ships as **0.4.0**, bundled with the terminal-state-sink fix already on
`main` (see `CHANGELOG.md` `[Unreleased]` and
`P1-ADR-TerminalStatesAreNotSinks`). The grant proposals bind "Phase 1 = 0.4.x on
crates.io", so a crates.io **publish** still happens — but only as a release-time
formality, once Phase 1 is complete.

Development does **not** wait for that publish. Phase 1 is built step-by-step on
`feat/phase-1-joining`, and Raft and IBFT consume it **directly by local path
reference** — no push, no publish, no git pin. Each consumer swaps its crates.io
pin (`faction = "0.3.2"`) for a path dependency on this working copy:

```toml
faction = { path = "../../faction/core" }
```

Consequences of the path model:

- Consumers build against the in-progress 0.4.0 line and co-evolve their
  integration **in lockstep** with Phase 1 — there is no publication gate.
- Because `main` (and this branch) already removed `Conclusion::TimedOut`, the
  forced compile-break migration lands **immediately** when the path is switched,
  not at some later re-pin: **Raft 2 sites**, **IBFT 1 site** (detailed in each
  repo's Faction-Phase-1 doc,
  `etheram-raft/docs/FACTION-PHASE-1-DESIGN.md` and
  `etheram-ibft/docs/FACTION-PHASE-1-INTEGRATION.md`). That migration is step 0 of
  each consumer's work.
- At publish time, consumers may flip the path dep back to a crates.io
  `version = "0.4.0"` pin, or keep the path for local dev. The published artifact
  is what satisfies the grant milestone.

### Future consumers (no assumptions)

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

### Decisions to lock before the first failing test

1. (decided) Member-set representation: **A — carried in each `State`** via a
   `Members` value object seeded from genesis.
2. (decided) The 7 join scenarios in a new `joining_tests.rs`, run
   **minimal-first** on Task/Thread × In-Memory; other transports + the Process
   spawn deferred until the core and minimal matrix are green.
3. (decided) Join-edge semantics — the **permissive/stateless** package: forward
   `JoinRequested` even for an existing member; re-approve → `DuplicateMemberIgnored`;
   **approve-without-prior-request is a stateless admit** (Faction holds no
   pending-request state); `JoinRejected` never removes (removal is Phase 4);
   self-join folds into the member rules; the quorum threshold stays fixed as the
   set grows; admission is not retroactive (the peer re-sends). This keeps Phase 1
   a pure gatekeeper — no coordination state (→ Phase 3), no removal (→ Phase 4),
   no quorum re-evaluation (→ the quorum question), no bounded queue (→ Phase 6),
   no signal buffering.
4. (decided) Consumer sequencing: **local path reference**, co-developed in
   lockstep; the crates.io publish is a release-time formality.
5. Raft/IBFT integration co-develops **in lockstep** via the path reference — the
   publish dependency that would have made it a follow-on is gone.
6. (open, Phase 1) `Collecting`'s response to a **member's** `ParticipationObserved`:
   re-advertise (`AcknowledgeRejoin`, stay) vs. accept/track. Surfaced by the IBFT
   integration; see the subsection above.

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

- *(written — `P1-ADR-ConfigIsImmutableGenesis`)* `Config` immutability,
  **membership half**: the live member set is state, grown only by the
  `JoinApproved` `Command`, never a setter; `Config` stays immutable genesis. The
  **quorum-threshold** half stays deferred on the quorum-change question above —
  the ADR is scoped to membership and explicitly carves the threshold out.
- *(written — `P2-ADR-ClusterViewBuilderAndDto`)* The consumer read-model is a pure
  DTO; construction lives in a separate `ClusterViewBuilder`, and live membership is
  observable on the view. Shipped on `feat/phase-1-joining`.
- *(written — `P2-ADR-TestingLadder`)* The test-tier ladder — unit, transition-matrix,
  property-based, cluster-simulation, system — and the `run_stage_1` + `run_stage_2`
  gate that runs them.
- *(deferred to post-Phase-6)* `PeerId` genericization (currently `u64`).
