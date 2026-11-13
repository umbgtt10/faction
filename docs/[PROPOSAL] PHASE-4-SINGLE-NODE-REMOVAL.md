# [PROPOSAL] Phase 4 — single-node removal

**Status:** accepted, 2026-11-13: every decision, D1 to D11; D5's count left to the caller, confirmed at the owner's word.
**Releases:** specification 0.4.0, then the crate 0.8.0.
**Order:** after Phases 2 and 3, on which it depends; before Raft's slice 5, which wires it as slice 4 wired the
addition, and before Phase 5.

`faction` answers: *"Can we shrink by one, without breaking quorum?"*

---

## 1. What Phase 4 adds

Phase 2 suspects and never removes: a suspect stays a member, and the quorum is counted with it. Phase 3 grows a
concluded cluster by one through the layer that commits changes above it. Phase 4 is the other half of Phase 3: a
concluded cluster shrinks by one, through the same layer, one change in flight at a time — and a node refuses to
coordinate a removal that would leave fewer members not suspected than the required count.

`faction` still decides nothing about *who*: it never removes a member for being suspected, and a deployment that
wants suspects removed asks for it (`P0-ADR-ProtocolAgnostic`). It sequences, refuses what would lose the quorum, and
tells the caller what to do next. How a change commits stays the caller's.

The roadmap's draft predates Phases 2 and 3, and this proposal settles what it left open:

| the roadmap's case | in Phase 4 |
|---|---|
| remove a suspected-dead node, `Degraded` | valid: a suspect's removal is never refused for the quorum (D3) |
| remove the last node in a minority partition, `Degraded` | `RemovalRefused`: it would leave fewer members not suspected than the count (D3) |
| a probe arrives after the node is removed | `NonMemberIgnored`, Phase 2's `liveness-non-member` row, unchanged (D10) |
| `RejectRemove { reason }` with `WouldBreakQuorum`, `NodeAlreadySuspected`, `ReconfigurationInFlight` | in turn: `RemovalRefused`; dropped; `Rejected` (D4) |

`Degraded` is not a stage: Phase 2 put liveness in the view (`P1-ADR-LivenessReportedNotStaged`). It reads as a
concluded node with suspects, its quorum `Held` or `Lost`.

## 2. The flow

On the node that coordinates the removal:

```
Bootstrapped ──RemovalRequested{p}──▶ Reconfiguring(removal of p) → ProposeRemoval{p}
Bootstrapped ──RemovalRequested{p}──▶ Bootstrapped                → RemovalRefused{p}       p alive, too few without it
Reconfiguring ──RemovalCommitted{p}──▶ Bootstrapped, p no member  → MemberRemoved{p}
Reconfiguring ──RemovalAborted{p}──▶ Bootstrapped, p a member     → RemovalWithdrawn{p}
```

On every node, the coordinator among them, whatever its stage: `RemovalCommitted{p}` makes `p` no member, as the layer
above applies the committed change everywhere. On `p` itself, if it is running and hears of it, the commit removes the
node from its own cluster: it is `Removed` (D7).

There is no handshake. The newcomer of an addition acknowledges its membership before it is counted, because a member
that cannot take part must never be counted; a member being removed has nothing to acknowledge, and the member most
worth removing is the one that cannot answer. The caller proposes the removal as soon as `faction` says
`ProposeRemoval`.

## 3. Decisions

**D1 — A removal is coordinated as an addition is, without the handshake.** `RemovalRequested` is admitted in
`Bootstrapped` alone; the coordinator is `Reconfiguring` while the removal is in flight, its peer state still
`Bootstrapped` (`P1-ADR-AdditionThroughTheLayerThatCommits`). Any concluded member may coordinate a removal, its own
included: a consensus leader may remove itself (D7). A peer that is not a member is answered `NonMemberIgnored`.
*Alternatives:* removals requested in every stage — refused: before the conclusion no layer above commits anything,
and a genesis peer that never starts is waited around by lowering the count (Phase 3's `RequiredCountChanged`); a
handshake with the member removed — refused for the reason given in §2.

**D2 — One change in flight, of either kind: the view's addition becomes its reconfiguration.** `Reconfiguring` holds
one change, an addition or a removal, and refuses a second request of either kind structurally — `Rejected`, the view
naming the change in flight (Phase 3's D3). The view's `addition` is renamed `reconfiguration`, with four values:
`None`, `AdditionRequested`, `AdditionAcknowledged` and `RemovalRequested`, each naming its peer.
*Alternative:* a second field, `removal`, beside `addition` — a smaller migration, but two fields for the one change a
node holds, and a pair of values the rules never reach that every reader must still handle.

**D3 — The quorum rule: a removal never takes the members not suspected below the count.** The removal of a member
not suspected is refused, `RemovalRefused`, when without it the members not suspected would number fewer than the
required count in effect. The removal of a suspect is never refused for the quorum: it leaves that number as it is.
The node counts itself and is never suspected (§14 of the specification), so a node removing itself is held to the
same rule. The coordinator decides on its own view, as each node judges liveness for itself (Phase 2's D6), at the
request; the commit is a fact of the layer above and is never refused (D6). On a node whose view differs — one that
still hears the member its coordinator suspects — a commit may lose the quorum, and says so: `MemberRemoved`, then
`QuorumLost`. No removal restores a quorum: it never adds to the members not suspected.
*Alternatives:* refuse every removal while the quorum is lost — refused: removing a suspect does not make a lost
quorum worse, and with a lowered count it is how a cluster that has lost members recovers; refuse a removal that
crosses the quorum from `Held` to `Lost` and allow the rest — refused: a node that has lost its quorum could then
remove its remaining live members one by one.

**D4 — The roadmap's three reasons, settled.** `WouldBreakQuorum` is `RemovalRefused`, the one reason a removal is
refused that is not a stage. `ReconfigurationInFlight` is the stage, and `Rejected` already says it, as Phase 3's D3
decided for a second addition. `NodeAlreadySuspected` is dropped: the roadmap's own table makes the removal of a
suspect valid, and it is what removal is mostly for. `RemovalRefused` carries no reason field: there is one reason,
and a later one would be an outcome of its own, as `AcknowledgementIgnored` and `AbortIgnored` are.

**D5 — The required count stays the caller's.** A removal changes who may be counted, never how many are required
(Phase 3's D5). The caller reports a new count with `RequiredCountChanged` when the change it belongs to commits —
before the removal, if it lowers the count, so that no node shows a quorum lost for the span between the two. A
removal that needs a lower count first is refused until the count is lowered.

**D6 — A commit is applied on every node, in every stage, and the peer leaves everything.** `RemovalCommitted` is a
fact the layer above reports, and every node applies it — one still bootstrapping, one restarting, the coordinator,
which then leaves `Reconfiguring` (Phase 3's D4). The peer leaves the members, the pinging peers, the collecting peers,
the suspected peers and the missed periods: a removed peer's signals no longer count, as a non-member's never have
(§6 of the specification). If it was the member probed last, the node has probed no one, and its next probe names the
first member that is not the node. A commit naming a peer that is not a member is `NonMemberIgnored`. A commit naming
another peer never moves a node out of `Initial`, as no membership decision does — only its own removal does (D7); a
commit naming another peer than the one in flight leaves the change in flight, as an addition's does.
*Alternative* for the member probed last: the member before it, so that the round-robin goes on exactly where it was —
refused: a rule with exceptions (the node itself, a lone member) for an evenness no reader depends on.

**D7 — A node's own removal: a sixth stage, `Removed`.** A commit naming the node itself is answered `MemberRemoved`,
and the node is `Removed`, its peer state `Removed`: it is no longer among its own members, it tracks no liveness —
no suspect, no missed period, no member probed last, its quorum `NotTracked` — and its change in flight, if any, ends
with it. It admits `Probe` alone, and it is not concluded: a caller that reads the peer state for the conclusion reads
a removed node as out. A way back is Phase 5's (epochs and rejoin, limitation L5).
`P1-ADR-TerminalStatesAreNotSinks` is amended: its rule keeps a node a peer waits on from going silent, and no member
waits on a removed node, since every member applies the same commit.
*Alternatives:* `SelfIgnored`, the node going on as a member of a cluster that no longer counts it — refused: it would
go on probing, suspecting and holding a quorum that means nothing; refusing a request that names the node itself —
refused: the commit can still come from a removal another node coordinated, so the stage is needed anyway, and a
leader removing itself is ordinary.

**D8 — Names in the specification's vocabulary.** The roadmap's drafts break both rules of §1 of the specification
(`RequestRemove`, `RemoveCommitted`, `RemoveAborted`, `RejectRemove`, `EmitMemberRemoved`). The commands are
`RemovalRequested`, `RemovalCommitted` and `RemovalAborted`; the outcomes `ProposeRemoval`, `RemovalRefused`,
`RemovalWithdrawn` and `MemberRemoved`. Reused: `NonMemberIgnored` (a peer that is not a member), `AbortIgnored` (an
abort that does not name the change in flight, of either kind), `AcknowledgementIgnored` (no addition awaits the
acknowledgement), `QuorumLost`. The full list is §4.

**D9 — Every 0.3.0 vector is migrated, its meaning unchanged.** As Phase 3's D7 and Phase 2's D10: the view's
`addition` becomes `reconfiguration`, its values renamed (`Requested` to `AdditionRequested`, `Acknowledged` to
`AdditionAcknowledged`); each stage's admissible list gains the new commands it admits. Every outcome and every other
field stays as it was, and the model checks the 78 migrated vectors before the crate runs them.

**D10 — Nothing new on the wire.** A removal needs no message of `faction`'s: the coordinator tells its caller, the
layer above reports the commit to every node it reaches, and the member removed is told by that layer or by no one. A
removed member's late answers are `PeerHeard` from a non-member, `NonMemberIgnored`. The protocol gains three entry
points and one output (§6).

**D11 — The system tests play the committing layer, as in Phase 3.** On `ProposeRemoval` the harness reports the
commit — and a lowered count first, where the scenario has one — to every running node, or the abort to the
coordinator. The scenarios remove a silenced member, refuse a removal that would lose the quorum, abort one, remove a
live member that then leaves, and remove a suspect with a lowered count (§7).

## 4. The specification, 0.4.0

**Commands** — after `AdditionAborted`, in the commands' order:

| command | carries | admitted in |
|---|---|---|
| `RemovalRequested` | `peer_id` | `Bootstrapped` |
| `RemovalCommitted` | `peer_id` | `Initial`, `Pinging`, `Collecting`, `Bootstrapped`, `Reconfiguring` |
| `RemovalAborted` | `peer_id` | `Reconfiguring` |

**Outcomes:** `ProposeRemoval { peer_id }` (propose the removal to the layer above), `RemovalRefused { peer_id }`,
`RemovalWithdrawn { peer_id }`, `MemberRemoved { peer_id }`; reused: `NonMemberIgnored`, `AbortIgnored`,
`AcknowledgementIgnored`, `QuorumLost`.

**Stage:** `Removed`, the sixth; its peer state `Removed`. **View:** `addition` becomes `reconfiguration` (D2).

**Admission** — each stage admits its 0.3.0 commands, and:

| stage | also admits |
|---|---|
| `Initial`, `Pinging`, `Collecting` | `RemovalCommitted` |
| `Bootstrapped` | `RemovalRequested`, `RemovalCommitted` |
| `Reconfiguring` | `RemovalCommitted`, `RemovalAborted` |
| `Removed` | `Probe` alone |

**Rows** — 0.3.0's forty-six keep their order and meaning. Four change: `probed` and `not-admitted` gain `Removed`;
`acknowledgement-ignored` applies whenever no addition of that peer awaits its acknowledgement, a removal in flight
among those cases; `abort-ignored` covers both aborts, applying when the abort does not name the change in flight. New
rows, after `addition-committed`, in check order:

| row | stages | command | when | result | next |
|---|---|---|---|---|---|
| `removal-non-member` | Bootstrapped | `RemovalRequested` | the peer is not a member | `NonMemberIgnored` | — |
| `removal-refused` | Bootstrapped | `RemovalRequested` | the peer is not suspected, and without it the members not suspected would number fewer than the required count | `RemovalRefused` | — |
| `removal-requested` | Bootstrapped | `RemovalRequested` | | `ProposeRemoval` | Reconfiguring |
| `removal-aborted` | Reconfiguring | `RemovalAborted` | | `RemovalWithdrawn` | Bootstrapped |
| `commit-non-member` | every stage but Removed | `RemovalCommitted` | the peer is not a member | `NonMemberIgnored` | — |
| `node-removed` | every stage but Removed | `RemovalCommitted` | the peer is the node itself | `MemberRemoved` | Removed |
| `removal-completion-loses-quorum` | Reconfiguring | `RemovalCommitted` | the peer of the removal in flight, and the quorum held is lost without it | `MemberRemoved`, `QuorumLost` | Bootstrapped |
| `removal-completed` | Reconfiguring | `RemovalCommitted` | the peer of the removal in flight | `MemberRemoved` | Bootstrapped |
| `removal-loses-quorum` | Bootstrapped, Reconfiguring | `RemovalCommitted` | the quorum held is lost without the peer | `MemberRemoved`, `QuorumLost` | — |
| `removal-committed` | every stage but Removed | `RemovalCommitted` | | `MemberRemoved` | — |

Ten new rows, 56 in all. `removal-aborted` follows `abort-ignored`, which has caught every abort not naming the
removal in flight.

**Prose:**
- §1: the list of commands;
- §2: six stages, members added and removed, `Removed` — leaving the cluster, and no command takes it anywhere;
- §4: the view's reconfiguration, the peer state of `Removed`, a removed node's quorum `NotTracked`;
- §5: admission;
- a new §12 *Removal*, the later sections renumbered: §13 *The required count*, §14 *Liveness* — a removed member's
  liveness goes with it, and the member probed last (D6) — §15 *Quorum*, where a removal may lose the quorum and
  never restores it, and §16 *Conformance*.

**Schema:** the commands, the outcomes, the stage, the peer state and the view's field. **Vectors:** at least one per
new row and per changed row's new case — 14 or more, about 92 in all — and the migration of D9.

## 5. The crate, 0.8.0

`Removed` is a state struct of its own behind `Stage` (`P2-ADR-StateStructsBehindStageEnum`); `Reconfiguring` holds
the change in flight, an addition or a removal. `Command` gains three commands and `Outcome` four outcomes; `PeerState`
gains `Removed`; `faction::addition::Addition` becomes `faction::reconfiguration::Reconfiguration`, and `ClusterView`'s
`addition` its `reconfiguration`, the builder's `with_addition` its `with_reconfiguration`. The members, the liveness
and the bootstrap lists each learn to forget a peer. All of it is breaking, hence 0.8.0.

The ladder follows the table: the matrix's cells for the sixth stage and the three commands — every `(stage,
command)` cell, the roadmap's gate — the property tier against the model, with an invariant that a removal is proposed
only of a suspect, or of a member without whom the members not suspected still number the count; the replay tier
unchanged in shape. Both coverage gates stay at 100%, every new line reached by the vectors alone.

## 6. The protocol

**Entry points**, beside Phase 3's: `request_removal(peer_id)`, `commit_removal(peer_id)` and
`abort_removal(peer_id)`, each one command.

**Output:** `ProposeRemoval { peer_id }`, for the committing layer.

**Translation.**
- `ProposeRemoval { p }` becomes `ProposeRemoval { p }`.
- `MemberRemoved { p }` cancels `p`'s `ProbeMissed` and `IndirectProbesMissed`, as `AliveNoted` does: no timer fires
  for a member that is gone. Were one to fire, it would be `NonMemberIgnored`.
- `RemovalRefused`, `RemovalWithdrawn` send nothing.

**A removed node** stops on its own: its probe period is scheduled again only while it is concluded, and `Removed`
is not, so the next `ProbeDue` ends it; any other timer fires into `Rejected`. Its `ProbeRelay` keeps answering what
reaches it, as it holds no table — and nothing reaches it from a member that has applied the commit, which neither
probes it nor lists it to help.

**The membership** a coordinator sends a newcomer is read from its view, so an addition after a removal hands the
newcomer the members without it, with no change.

**Tests** (`crates/protocol/tests/`, written first): each entry point; each translation; a removed node's probe period
ending; an addition after a removal. Coverage stays at 100%.

## 7. The system tests

### Two tiers

**`validation/protocol-validation`**, the deterministic in-process cluster with drop rules and held liveness timers,
carries what a real transport cannot force:
1. a suspect is removed: silenced, its timers elapsed until it is suspected, its removal proposed and committed; every
   node shows the members without it, the quorum held;
2. a removed member's late answer, held back until after the commit, is ignored: no revival;
3. a removal committed during the bootstrap, its lowered count first, lets the rest conclude without the member;
4. a node removed stops probing: its own removal committed, it is `Removed` and no probe of its leaves after it;
5. a commit loses the quorum on a node whose view differs: the coordinator, its link to the member cut, suspects it;
   another node still hears it, and reports `QuorumLost` on the commit;
6. a removal in flight refuses an addition, and an addition in flight a removal;
7. an abort withdraws the removal: the member stays, on every node.

**`validation/system-tests`** runs each scenario in all ten spawn and transport cases — `Task` and `Thread` over
in-memory, channels, TCP and gRPC; `Process` over TCP and gRPC — the liveness cases' `ProbeStretch` where a scenario
waits for a suspicion:

| scenario | asserts |
|---|---|
| `a_silenced_member_is_suspected_then_removed` | four nodes, count three: the silenced member suspected, then removed; three members on every other node, their quorum held |
| `a_removal_that_would_lose_the_quorum_is_refused` | three nodes, count three: the coordinator refuses a live member's removal; three members everywhere, no change in flight |
| `an_aborted_removal_keeps_the_member` | the removal proposed, then aborted; four members everywhere, no change in flight |
| `a_removed_node_leaves_the_cluster` | a live member removed: it is `Removed`, the rest show three members; silenced afterwards, it is suspected by no one |
| `a_committed_removal_lowers_the_required_count` | three nodes, count three, one silenced: the quorum lost; the count lowered to two and the suspect removed; two members and the quorum held everywhere |

Fifty new system tests, the in-memory deployment among them in every scenario.

### The harness

- **The committing layer:** `Cluster::request_removal(coordinator, peer)`, `poll_until_removal_proposed(coordinator,
  peer)` — read from the coordinator's view, as Phase 3's proposed addition is — `commit_removal(peer, required_count)`,
  the count first if one is given, to every running node, each report waiting for its acknowledgement; and
  `abort_removal(coordinator, peer)`.
- **`FactionNode`** dispatches `ProposeRemoval` where the harness reads it, and re-reads its members from its view after
  a removal, as after an admission or an addition; a removed node broadcasts to no one.
- **`Node`, `NodeSnapshot` and the process:** the thread and stdin commands gain `request-removal`, `commit-removal` and
  `abort-removal`; the snapshot's addition becomes its reconfiguration; the state line keeps its tokens and gains
  `removing:N` and the peer state `Removed`.
- **The wire:** unchanged — no new message, no new `MessageKind`.

### Order

1. Protocol tests, seen failing.
2. `protocol-validation` scenarios, seen failing.
3. The system scenarios last, seen failing until the harness carries them.

## 8. The independent model

`crates/spec-model/` follows the specification before the crate does: the stage, the three commands and the four
outcomes in `vocabulary/`; each new row's situation and change in the rule book — a peer not a member, the node
itself, a live member the quorum cannot spare, the removal in flight, the quorum lost on the commit — and the change in
flight in its node state. The 0.4.0 vectors, new and migrated, pass against it before the crate is touched.

## 9. Order of work

1. The specification 0.4.0: prose, schema, matrix, the new vectors, the migration.
2. The model, until every vector passes against it.
3. The crate, test first, until the conformance tier passes; then the matrix and property tiers.
4. The protocol, then `protocol-validation`, then the system tests (§6, §7).
5. ADRs (a Phase 4 ADR, `P1-ADR-RemovalThroughTheLayerThatCommits`; `P1-ADR-TerminalStatesAreNotSinks` amended for
   `Removed`), ROADMAP (Phase 4 complete; names as §4), ARCHITECTURE (L4 gone; members added and removed), CHANGELOG,
   README.
6. Release 0.8.0 at the owner's word.

## 10. Out of scope

A way back for a removed node, and refusing a removed peer's stale identity (Phase 5): until then a removed peer's id
may be added again as any non-member's, but a `Removed` node cannot take the membership. Concurrent changes, and a
queue of them (Phase 6). Removing a member for its suspicion, on any policy: the caller's. Computing any count;
deciding who may be removed; how a change commits.
