# [PROPOSAL] Phase 6 — concurrent membership changes

**Status:** accepted 2026-11-17, at the owner's word: Phase 6 is **A** — a proposal carries the epoch it was decided
at, and the layer above commits it only while it is still valid — and **C** — a coordinator holds several changes in
flight; the ROADMAP's queue (`EnqueueAdd`, `Dequeued`, `QueueFull`, `FlushQueue`) is dropped; Phase 6 is released only
after a 12-hour soak with no failure, and not yet as 1.0. Every decision of §10 as proposed: the conflict-aware commit
rule, `ProposalDeclined`, the stage `Coordinating` and the view's reconfigurations as a list, and the bound
`changes_in_flight`, 1 unless stated.

**Releases:** the specification's 0.9.0; the crate's 0.13.0, once the 12-hour soak has passed. 1.0 is later, at the
owner's word.
**Order:** after 0.12.0 and the soak fixes.

Phase 6 answers: *"Can several membership changes be in flight at once — and is every one committed safe by
construction, not by the caller's diligence?"*

---

## 1. What it adds

- **A proposal states the epoch it was decided at**, and the specification states what the layer above owes it: to
  commit it only while no change committed since conflicts with it (A).
- **A declined proposal is decided again**, on the coordinator's present view: proposed once more, or given up.
- **A coordinator may hold several changes in flight**, up to a bound its configuration states, its removals together
  sparing the quorum (C).
- **The chaos tier lets conflicting proposals race**, the commit rule of A the only thing between them and a wrong end
  state.

## 2. Where it starts

**Changes already run concurrently across the cluster.** Each coordinator holds one change in flight; several
coordinators hold one each; the layer above commits them in epoch order, and every node applies every commit, in any
stage, a change of its own in flight or not.

**But a change is checked against its coordinator's view when it is requested, and committed later.** Nothing in the
specification says the layer above must check it again. Two removals each sparing the quorum on its own commit
together below the required count — the node applies both and reports `QuorumLost`; an addition and a removal of one
peer both commit, and the later wins. Today a change is safe only because the layer above checks it again at commit
time: the chaos tier's does (`LayerAbove::keeps_the_log_valid`), and every real consumer must write its own, unstated.

**And a coordinator holds one change at a time.** `Reconfiguring` refuses a second request; growing a cluster by many
nodes goes one addition per coordinator per round of consensus.

**The specification grows by its rule.** No released answer may change: a new configuration member is omitted at the
value that keeps the old behaviour, a new command is named in no released vector's admissible commands, and a stage, an
outcome, a command may be added (§1). Every decision below keeps to it.

## 3. A: a proposal's epoch, and what the layer above owes it

**D1. A proposal is decided at the epoch its answer's view shows.** `ProposeAddition` and `ProposeRemoval` keep their
form — an epoch in the outcome would change every released vector that proposes after a commit — and the view the
answer carries already states the epoch. faction-protocol's `OutputMessage::ProposeAddition` and `ProposeRemoval` carry
it explicitly, for the caller to submit with the change.

**D2. What the layer above owes a proposal: to commit it only while it is still valid.** A proposal decided at epoch
`e` is valid at the layer above's epoch `c` when no change committed in `(e, c]` conflicts with it:

- a change naming the same peer conflicts with it;
- for a removal, another removal or a change of the required count conflicts with it too — either moves what the
  removal's quorum check counted;
- an addition never conflicts with a removal of another peer: it only adds to the members the removal spares.

The rule reads the committed log alone, never a node's suspicions, so any layer above can keep it. *The alternative,
strict:* valid only if nothing at all has committed since `e`. Simpler to keep, but it serializes what runs in parallel
today — two additions of different peers, from two coordinators, would make one of them wait a round.

**D3. A proposal the layer above declines is decided again.** A new command, `ProposalDeclined { peer_id, epoch }`,
reports to the coordinator a proposal declined at the layer above's epoch. The coordinator decides the change again
on its present view: valid still, it is proposed again, at its present epoch; no longer valid — a peer added already, a
removal the quorum can no longer spare — it is given up, `AdditionWithdrawn` or `RemovalRefused`, and the node holds it
no more. *The alternative:* the layer above reports `AdditionAborted` or `RemovalAborted`, and the caller asks again —
for an addition, a whole round trip to the newcomer once more, its acknowledgement already given.

## 4. C: several changes in flight

**D4. The bound is the configuration's.** The configuration states how many changes the node may hold in flight as a
coordinator, `changes_in_flight`, 1 unless it states another — omitted at 1, so every released vector is answered
alike. The caller chooses it, as the deadline and the suspicion threshold are the caller's.

**D5. A stage below the bound.** A node with changes in flight and room for more is `Coordinating`, a new stage: it
admits a request of either kind, and everything `Reconfiguring` admits. A node at its bound is `Reconfiguring`, which
admits no request, as now. With a bound of 1 no node is ever `Coordinating`. The view shows both as `Bootstrapped`, as it
shows `Reconfiguring` now.

**D6. The view lists every change in flight.** `reconfiguration` is one value today. *Recommended:* re-encode it once, as
0.8.0 re-encoded the vectors — `reconfiguration` becomes `reconfigurations`, a list in the order requested, empty with
nothing in flight — and `ClusterView::reconfigurations()` replaces `reconfiguration()`. No answer changes. *The alternative:* keep `reconfiguration`, the first change in flight, and add the list beside it, omitted with at
most one — no re-encoding, two members saying one thing.

**D7. A coordinator's changes together.** Two changes in flight never name the same peer: a request naming a peer in
flight is answered as a duplicate is. A removal is refused when, with every removal in flight, the members not
suspected would number fewer than the required count — the quorum check counts them together. An addition and a
removal stand together.

**D8. Each change ends on its own.** A commit naming a peer in flight ends that change, the others staying; an abort
names its peer; an acknowledgement is the awaited one if its newcomer's addition is in flight; `RetryMembership` is per
newcomer already. An adopted membership gives up every change in flight, a `ChangeAbandoned` for each. A node with
changes in flight still admits no join decision: no admission goes around a change in flight.

## 5. The specification, 0.9.0

- §1 notes the growth, §2 the bound, §4 the view's list (D6), §5 `Coordinating`'s admissions, §11–§12 the changes
  together, a new section on what the layer above owes a proposal (D2) and `ProposalDeclined` (D3), §16 an adoption
  abandoning every change.
- The matrix: `Coordinating`'s rows; the request rows at the bound and below it; the removal's quorum check over every
  removal in flight; `ProposalDeclined`'s rows.
- Vectors: two additions in flight and their commits in either order; a removal refused for the removals in flight; a
  proposal declined and proposed again, and one given up; an adoption abandoning two changes; and every released vector
  unchanged, the bound omitted at 1.

## 6. The crate and the protocol

- `Config` takes the bound; `Stage` gains `Coordinating`; the reconfiguration becomes a list (D6); `Command` gains
  `ProposalDeclined`. Breaking.
- faction-protocol: `ProposeAddition` and `ProposeRemoval` carry their epoch; a declined proposal reaches the machine as
  `ProposalDeclined`.

## 7. Validation

- **The ladder:** conformance and the matrix for every new row, the model first; properties over random commands with
  bounds above 1 — a node never holds two changes naming one peer, never more than its bound, and its removals together
  always spare the quorum when it requests them.
- **The chaos tier:** scenarios draw a bound per coordinator, from 1 to 3; the layer above lets every proposal race and
  commits by D2 alone, `keeps_the_log_valid` retired as the guard it was. A new invariant: the rule never admits a
  change the committed log cannot hold — an addition of a member, a removal of a peer that is none — and a run
  that would fails. *Agreed with the owner instead of the first draft's "the committed log never takes the members
  below the required count":* that one is not the specification's to keep, since a coordinator that has lost its
  quorum may remove a suspect (§12, vector 86). The oracle's end state is unchanged.
- **The release gate:** a 12-hour soak, no failure; then 0.13.0.

## 8. Out of scope

- **Batched changes**, several additions and removals committed at one epoch: a later phase, if a protocol asks.
- **The queue** of the ROADMAP's draft: dropped — ordering is the layer above's.
- **A lying peer:** the Notary's.

## 9. Order of work

1. This proposal, accepted.
2. A: the specification's D1–D3, the model, the crate, the protocol; the chaos layer above committing by D2.
3. C: the bound, `Coordinating`, the list, D7–D8; the chaos scenarios' bounds.
4. The docs; the ROADMAP's Phase 6 rewritten.
5. The 12-hour soak; then 0.13.0, at the owner's word.

Each step on the branch `phase-6`, both gates green on every commit.

## 10. For the owner to decide

1. **D2:** the conflict-aware commit rule, keeping today's parallel changes — or strict, nothing committed since.
2. **D3:** a declined proposal decided again by the machine, `ProposalDeclined` — or aborted, and asked again by the
   caller.
3. **D5 and D6:** a new stage, `Coordinating`, below the bound; and the view's reconfiguration re-encoded as a list —
   or kept, with the list beside it.
4. **The name of the bound**, `changes_in_flight`, and its default, 1.
