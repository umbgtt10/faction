# [PROPOSAL] Phase 3 — single-node addition

**Status:** decisions D1–D9 accepted by the owner, 2026-11-13. The plan for the protocol and the
system tests (§6, §7) awaits the owner's word; nothing is built.
**Releases:** specification 0.2.0, then the crate 0.6.0.
**Order:** Phase 3 goes ahead of Phase 2 (the owner's order); it depends on Phase 1 alone.

`faction` answers: *"Can a running cluster grow by one, safely?"*

---

## 1. What Phase 3 adds to Phase 1

Phase 1 admits a peer: `JoinApproved` makes it a member in any stage, at once, with no
coordination and no way back. It answers *who may be counted*, and it stays exactly as it is —
every 0.1.0 row keeps its meaning.

Phase 3 changes the membership of a **bootstrapped** cluster through the layer that commits
changes above it: one addition in flight at a time, the newcomer handed the membership before it
is counted, and the change taking effect only when the layer above says it committed — or
withdrawn when it says it aborted. `faction` still decides nothing: it sequences, refuses a
second change while one is in flight, and tells the caller what to do next. Who may be added,
and how a change commits, are the caller's (`P0-ADR-ProtocolAgnostic`).

## 2. The flow

On the node that coordinates the addition:

```
Bootstrapped ──AdditionRequested{p}──▶ Reconfiguring(p, unacknowledged)    → SendMembership{p}
Reconfiguring ──MembershipAcknowledged{p}──▶ Reconfiguring(p, acknowledged) → ProposeAddition{p}
Reconfiguring ──AdditionCommitted{p}──▶ Bootstrapped, p the last member     → MemberAdmitted{p}
Reconfiguring ──AdditionAborted{p}──▶ Bootstrapped, nothing added           → AdditionWithdrawn{p}
```

On every node, the coordinator among them, whatever its stage: `AdditionCommitted{p}` makes `p`
a member, as the layer above applies the committed change everywhere.

The acknowledgement is the newcomer saying it holds the membership: a member that cannot yet
take part is never counted. The caller proposes the addition to the layer above only once
`faction` says `ProposeAddition`.

## 3. Decisions

**D1 — Additions only after bootstrap; Phase 1 untouched.** `AdditionRequested` is admitted
in `Bootstrapped` alone. Before bootstrap there is no running cluster to protect, and Phase 1's
admission is the way in. `JoinApproved` keeps every 0.1.0 row, `Bootstrapped` included.
*Alternative:* additions in every stage — refused: before bootstrap no layer above commits
anything yet.

**D2 — `Reconfiguring`, a fifth stage, shown as `Bootstrapped`.** The addition in flight is a
stage, so admission refuses a second one structurally (§5 of the specification), as the roadmap
asks. Its peer state stays `Bootstrapped` — a reconfiguring node is still concluded, and a
caller that reads concluded from the peer state reads it unchanged — and the view gains the
pending addition: the peer and whether it has acknowledged. `Bootstrapped` stays the only
conclusion; "no command leaves it" becomes "no command leaves the concluded stages".
*Alternatives:* the pending addition as a flag inside `Bootstrapped`, refused because admission
is by stage and a flag would need its own refusal rows; a peer state of its own, refused because
every caller reading `Bootstrapped` as concluded would break.

**D3 — A second request is `Rejected`.** `Reconfiguring` does not admit `AdditionRequested`: the
result is `Rejected`, its view naming the addition in flight. No `RejectAdd { reason }` outcome —
the one reason Phase 3 has is the stage, and `Rejected` already says it. Phase 4's reasons
(a removal that would break quorum) are not a stage, and will be outcomes then.

**D4 — A commit is applied on every node, in every stage.** `AdditionCommitted` is a fact the
layer above reports, and every node applies it — a node that coordinated nothing, a node still
bootstrapping or restarting, the coordinator, which then leaves `Reconfiguring`. A commit naming
a member is `DuplicateMemberIgnored`, as a duplicate admission is.
*Alternative:* commits admitted on the coordinator only — refused: every other node would have
to fake a Phase 1 admission to learn the same fact.

**D5 — The required count changes by a command of its own.** `RequiredCountChanged
{ required_count }`, admitted in every stage, sets the count; the caller computes it, as at
creation, and reports it when the change carrying it commits. In `Collecting`, a count the
collecting peers already reach concludes, as readiness does. This is the command
`OPEN_POINTS.md` asks for — a command through admission and the decision table, never a setter —
the coordination it lacks coming from the commit it is reported with. The configuration stays
immutable; the count in effect becomes state, starting at the configuration's
(`P1-ADR-ConfigIsImmutableGenesis`, amended).
*Alternatives:* the count fixed — the smallest Phase 3, but a grown cluster keeps its genesis
count and the open point stays open; the count carried by `AdditionCommitted` — one call per
commit, but a commit naming a member already admitted would then have to change the count and
not the members, and Phase 4 would carry it again.

**D6 — Names in the specification's vocabulary.** Commands say what the caller observed,
outcomes what happened or what to do (§1 of the specification); the roadmap's drafts break both
(`RequestAdd`, `JoinAck`, `EmitMemberAdded`). Phase 1's `MemberAdmitted` and
`DuplicateMemberIgnored` are reused: their effect is the same. The full list is §4.

**D7 — Every 0.1.0 vector is migrated, its meaning unchanged.** A result carries the view and the
admissible commands, and both grow: the view by the pending addition (none, in every 0.1.0
step), each stage's admissible list by the new commands it admits. So all 41 vectors change —
mechanically, in those two places only, every outcome and every other field of the view as it
was — and the model checks them before the crate runs them. "Phase N tests pass unchanged" holds
for behaviour; a result's self-description cannot stay unchanged while the vocabulary grows.
*Alternative:* leave the pending addition out of the view — refused: a vector checks state the
way a caller would, and could not see the addition in flight.

**D8 — The system tests play the layer that commits.** `faction` never commits, so the system
tier needs something that does: the harness, as its `Approver` already decides joins. On
`ProposeAddition` it reports `AdditionCommitted` (and the new count) to every node, or
`AdditionAborted` to the coordinator in the abort scenarios. The protocol carries the two new
messages the newcomer exchanges with the coordinator — the membership, and its acknowledgement.

**D9 — `join-raises-quorum` returns as an addition.** The postponed Phase 1 scenario presupposed
a coordinated count change; with D4 and D5 it is "a committed addition raises the required
count", run across the spawn/transport matrix, and the open point closes.

## 4. The specification, 0.2.0

**Commands** — in the commands' order, after 0.1.0's:

| command | carries | admitted in |
|---|---|---|
| `AdditionRequested` | `peer_id` | `Bootstrapped` |
| `MembershipAcknowledged` | `peer_id` | `Reconfiguring` |
| `AdditionCommitted` | `peer_id` | every stage |
| `AdditionAborted` | `peer_id` | `Reconfiguring` |
| `RequiredCountChanged` | `required_count` | every stage |

**Outcomes:** `SendMembership { peer_id }` (send the newcomer the membership: the members and the
required count), `ProposeAddition { peer_id }` (propose the addition to the layer above),
`AdditionWithdrawn { peer_id }`, `AcknowledgementIgnored { peer_id }` (a peer that is not the
pending one, or a repeat), `AbortIgnored { peer_id }`, `RequiredCountSet { required_count }`;
reused: `MemberAdmitted`, `DuplicateMemberIgnored`, `Concluded`.

**Admission** — each stage admits its 0.1.0 commands, and:

| stage | also admits |
|---|---|
| `Initial`, `Pinging`, `Collecting` | `AdditionCommitted`, `RequiredCountChanged` |
| `Bootstrapped` | `AdditionRequested`, `AdditionCommitted`, `RequiredCountChanged` |
| `Reconfiguring` | `ParticipationObserved`, `JoinRequested`, `JoinRejected`, `MembershipAcknowledged`, `AdditionCommitted`, `AdditionAborted`, `RequiredCountChanged`, `Probe` |

`Reconfiguring` does not admit `JoinApproved`: no uncoordinated admission while a coordinated one
is in flight.

**Rows** — 0.1.0's eighteen keep their order and meaning; those that apply in `Reconfiguring`
gain it (`probed`, `not-admitted`, `non-member`, `rejoin-acknowledged`, `join-requested`,
`join-denied`). New rows, in check order:

| row | stages | command | when | result | next |
|---|---|---|---|---|---|
| `addition-member` | Bootstrapped | `AdditionRequested` | the peer is a member | `DuplicateMemberIgnored` | — |
| `addition-requested` | Bootstrapped | `AdditionRequested` | | `SendMembership` | Reconfiguring |
| `acknowledgement-ignored` | Reconfiguring | `MembershipAcknowledged` | not the pending peer, or already acknowledged | `AcknowledgementIgnored` | — |
| `acknowledgement-accepted` | Reconfiguring | `MembershipAcknowledged` | | `ProposeAddition` | — |
| `abort-ignored` | Reconfiguring | `AdditionAborted` | not the pending peer | `AbortIgnored` | — |
| `addition-aborted` | Reconfiguring | `AdditionAborted` | | `AdditionWithdrawn` | Bootstrapped |
| `commit-member` | every stage | `AdditionCommitted` | the peer is a member | `DuplicateMemberIgnored` | — |
| `addition-completed` | Reconfiguring | `AdditionCommitted` | the pending peer | `MemberAdmitted` | Bootstrapped |
| `addition-committed` | every stage | `AdditionCommitted` | | `MemberAdmitted` | — |
| `count-concluded` | Collecting | `RequiredCountChanged` | the collecting peers reach it | `RequiredCountSet`, `Concluded` | Bootstrapped |
| `count-set` | every stage | `RequiredCountChanged` | | `RequiredCountSet` | — |

A commit for a peer other than the pending one adds it and leaves the addition in flight.
Membership commands never move a node out of `Initial`, as join decisions do not.

**Prose:** §2 (five stages; the count in effect is state), §4 (the view's pending addition; the
peer state of `Reconfiguring`; the required count in effect), §5 (admission), a new §11
*Addition* and §12 *The required count*, §1's list of commands. **Schema:** the commands,
outcomes, stage and view field. **Vectors:** at least one per new row, and the migration of D7.

## 5. The crate, 0.6.0

`Reconfiguring` is a state struct of its own behind `Stage`
(`P2-ADR-StateStructsBehindStageEnum`), its steps on the punch model; `Command`, `Outcome` and
`ClusterView` grow (breaking, so 0.6.0); the required count in effect moves into the state, the
`Config` immutable. The ladder's tiers follow the table: the matrix's cells for the fifth stage
and the new commands, the property tier against the model, the replay tier unchanged in shape.
Both coverage gates stay at 100%: every new line reached by the vectors alone.

## 6. The protocol

`faction-protocol` turns outcomes into messages and timers. Today it only broadcasts — `Ping`,
`Ready`, `Bootstrapped` — and its translator stops at the first outcome it recognises.

**Two messages on the wire.**
- `Membership { from, members, required_count }`, coordinator to newcomer: the members and the
  required count in effect, read from the result's view.
- `MembershipAcknowledged { from }`, newcomer to coordinator.

Nothing else crosses the wire: a commit, an abort and a new count are the committing layer's,
reported to each node through the protocol's entry points, never a `faction` message.

**Outputs.**
- `Send { to, message }`: one message to one peer. The transport port already sends to one peer;
  the broadcasts stay as they are.
- `ProposeAddition { peer_id }`, for the committing layer.
- A timer, `RetryMembership { peer_id }`.

**Entry points**, beside `admit`, `deny`, `request_join` and `expire_deadline`:
`request_addition(peer_id)`, `commit_addition(peer_id)`, `abort_addition(peer_id)` and
`change_required_count(required_count)`, each one command.

**Translation.**
- `MembershipAcknowledged { from }` is the command `MembershipAcknowledged { peer_id: from }`.
- `SendMembership { p }` becomes `Send { to: p, Membership }`, built from the result's view, and
  schedules `RetryMembership { p }`.
- `ProposeAddition { p }` becomes `ProposeAddition { p }`, and cancels the retry.
- `AdditionWithdrawn { p }` cancels the retry, and so does a `MemberAdmitted { p }` that ends the
  addition.
- Every other new outcome sends nothing.

The translator maps **every** outcome of a sequence, in order, and needs the result's view to do
it. Today `[LocalParticipationCompleted, BroadcastLocalReady, Concluded]` yields the broadcast and
never cancels the retry timers: harmless, since a retry finds the node concluded and stops, but a
sequence of Phase 3's would lose its second half. This is a change of behaviour, tested first.

**Re-sending.** `RetryMembership { p }` re-sends the membership while the view's pending addition is
`p`, unacknowledged, and rescheduled; otherwise it does nothing and stops, as `RetryPing` and
`RetryReady` do. A lost membership is re-sent; a lost acknowledgement is answered again, by the
re-sent membership.

**The newcomer.** A newcomer has no configuration until the membership arrives. `Newcomer` holds
its id and the observer its node will have, and waits. Given a `Membership`, it becomes a
`Protocol` over a node created from that membership — its own id, the members, itself among them,
and the required count — and its first outputs are `Send { to: from, MembershipAcknowledged }`,
then `initialize()`'s. A running node answers every later `Membership` with the acknowledgement
again; the coordinator's node ignores a repeat. The membership says who; the addresses stay the
transport's.

**Tests** (`crates/protocol/tests/`, written first): each entry point; each translation; the
translator's whole sequences; the re-sending; the newcomer's handshake. Coverage stays at 100%.

## 7. The system tests

### Two tiers

**`validation/protocol-validation`** is the deterministic in-process cluster, with drop rules.
It carries the scenarios a real transport cannot force:
1. a lost membership is re-sent until acknowledged;
2. a lost acknowledgement is recovered from the re-sent membership;
3. a commit reaching a node still bootstrapping admits the newcomer there, and counts it;
4. a commit for a peer other than the pending one leaves the addition in flight;
5. a required count lowered in `Collecting` concludes;
6. an abort before the acknowledgement stops the re-sending.

Its `Cluster` gains newcomers, the four entry points, and the dispatch of `Send` and
`ProposeAddition`.

**`validation/system-tests`** runs each scenario in all ten spawn and transport cases —
`Task` and `Thread` over in-memory, channels, TCP and gRPC; `Process` over TCP and gRPC:

| scenario | asserts |
|---|---|
| `an_added_newcomer_is_committed_and_bootstraps` | every node shows four members; all four bootstrapped. The addition is coordinated by node 1, not node 0: any member may coordinate. |
| `an_aborted_addition_never_counts_the_newcomer` | three members everywhere; no addition pending on the coordinator |
| `a_second_addition_is_refused_while_one_is_in_flight` | the coordinator's pending addition stays the first; after the commit, the second succeeds |
| `a_committed_addition_raises_the_required_count` | the required count is three on every node, the newcomer among them, and every node bootstrapped (D9) |
| `additions_one_after_another_each_converge` | two newcomers added in turn; five members; all bootstrapped |

That is fifty new system tests, the in-memory deployment among them in every scenario.

### The harness

**The committing layer.** The harness plays it, as `Approver` decides joins today:
- `Cluster::request_addition(newcomer, coordinator)` builds the newcomer waiting and asks the
  coordinator;
- `poll_until_proposed(newcomer)` waits for the coordinator's `ProposeAddition`;
- `commit_addition(newcomer, required_count)` reports the commit, and a new count if one is given,
  to every node, the newcomer first. Members ignore the newcomer until they hold the commit, so it
  cannot conclude on the old count. Each report waits for the node's acknowledgement, as `admit`
  does today;
- `abort_addition(newcomer)` reports the abort to the coordinator;
- `Cluster::add(newcomer, Committer::Commit { .. } | Committer::Abort)` does all of it in one call.

**The newcomer** joins the transport as Phase 1's late joiner does: in process, through the late-join
mesh; as a process, launched with `--newcomer` and the genesis addresses, as today, but no peer list.

**`FactionNode`.**
- It broadcasts to the members its view shows, not to a list of its own. The view is then the one
  source, and a commit widens the broadcasts with no bookkeeping of the harness's. The members and
  the list are the same set today, so no existing scenario changes.
- It dispatches `Send`.
- It records `ProposeAddition` where the harness can read it.
- It holds a waiting `Newcomer` until its membership arrives.

**`Node` and `NodeSnapshot`.**
- The snapshot gains the required count and the pending addition: the peer, and whether it has
  acknowledged.
- The thread commands and the process's stdin commands gain `request-addition`, `commit-addition`,
  `abort-addition` and `required-count`.
- The process's `state` line gains the two fields.

**The wire.**
- TCP and gRPC each keep their own copy of today's 9-byte codec. They will share one codec for all
  five messages: a frame is its length, the sender, a tag and the payload; the membership's
  payload is the count and the members.
- TCP keeps a receive buffer per connection. Today the bytes of every connection go into one
  buffer, which works only because every frame is the same 9 bytes; a variable-length frame split
  across reads would interleave with another connection's.
- In-memory and channels pass the messages as values, unchanged.
- `FaultyTransport`'s `MessageKind` gains the two kinds.

Each change is tested first, in `tests/infra/`: a frame split across reads on two connections, and
every message round-tripping each codec.

**Logs** keep their shape; the new commands and results are written as the rest are.

### Order

1. Protocol tests, seen failing.
2. `protocol-validation` scenarios, seen failing.
3. The codec and the TCP buffer tests, seen failing.
4. The system scenarios last, seen failing until the harness carries them.

## 8. The independent model

`crates/spec-model/` follows the specification before the crate does: the stage, the commands
and the outcomes in `vocabulary/`, each new row's situation and change in the rule book, the
pending addition in its node state. The 0.2.0 vectors — new and migrated — pass against it before
the crate is touched.

## 9. Order of work

1. The specification 0.2.0: prose, schema, matrix, the new vectors, the migration.
2. The model, until every vector passes against it.
3. The crate, test first, until the conformance tier passes; then the matrix and property tiers.
4. The protocol, then `protocol-validation`, then the system tests (§6, §7).
5. ADRs (`P1-ADR-ConfigIsImmutableGenesis` amended; Phase 3's own), ROADMAP (Phase 3 depends on
   Phase 1; names as §4), ARCHITECTURE, CHANGELOG, README; OPEN_POINTS loses the quorum-size
   question.
6. Release 0.6.0 at the owner's word.

## 10. Out of scope

Liveness (Phase 2); removal (Phase 4); epochs and rejoin (Phase 5); concurrent changes
(Phase 6); computing any count; deciding who may be added; how a change commits.
