# [PROPOSAL] Phase 2 — failure detection

**Status:** accepted, 2026-11-13: every decision, D1 to D12, the owner's twelfth added at their word.
**Releases:** specification 0.3.0, then the crate 0.7.0.
**Order:** after Phase 3, before Phase 4, which removes a member and needs to know which members are alive.

`faction` answers: *"Is everyone still alive?"*

---

## 1. What Phase 2 adds

Once concluded, a node today knows who its members are and nothing about whether they are still there: the
machine is terminal for liveness (L2). Phase 2 has a concluded node keep asking: it names, one protocol period at a
time, the member to probe; a member that misses its probe is probed again through others; one that misses both,
in as many periods running as the node's suspicion threshold, is suspected; one heard from again is revived. The node tells the caller when its members not suspected fall below the
required count in effect, and when they reach it again.

`faction` still performs no I/O and keeps no clock (`P0-ADR-PureMealyNoIO`). It decides whom to probe, whom to ask,
when to suspect and when quorum is lost; the caller decides when a period starts, how long a probe may take, and how
probes travel, and reports what it saw. How tolerant a node is, is its deployment's: the timings and the helpers
asked are the caller's, and the suspicion threshold is given to the node when it is created (D12). It suspects; it never removes. A member leaves by removal alone — Phase 4,
through the layer that commits changes, as an addition joins (`P1-ADR-AdditionThroughTheLayerThatCommits`).

This is SWIM's failure detector — round-robin probing, indirect probes through other members, suspicion — without its
dissemination (D6) and without its confirmation of a suspect as failed (D5).

## 2. The flow

On a concluded node — `Bootstrapped` or `Reconfiguring` — every protocol period:

```
ProbeDue                 → SendProbe{p}                     p: the member after the last probed, round-robin
ProbeMissed{p}           → SendIndirectProbes{p, via}       via: the alive members after p, in member order
IndirectProbesMissed{p}  → MissCounted{p, misses}           below the threshold: p's misses grow by one
IndirectProbesMissed{p}  → PeerSuspected{p}  [QuorumLost]   at the threshold: p suspected
PeerHeard{p}             → AliveNoted{p}                    p alive: its misses back to none
PeerHeard{p}             → PeerRevived{p}    [QuorumRestored]
```

`QuorumLost` and `QuorumRestored` close an outcome sequence when, and only when, the command moved the members not
suspected across the required count. A new member, a new count or a revival can restore quorum; a suspicion or a
raised count can lose it.

## 3. Decisions

**D1 — Liveness is something the view reports, not a stage.** The roadmap's `Stable`, `Degraded(SuspectSet)` and
`QuorumLost` would be three new stages, and each would need a twin while an addition is in flight. Liveness is not a
step in bootstrapping or in an addition: it runs alongside both, as a missed deadline does
(`P1-ADR-TerminalStatesAreNotSinks`). The view gains the suspected members, the members with missed rounds and how many, the last member probed, and the
quorum:
`NotTracked` before the conclusion, then `Held` or `Lost`. The roadmap's three names become a way of reading the
view: no member suspected; some suspected, quorum held; quorum lost.
*Alternative:* three stages — refused: `Reconfiguring` would split three ways and so would every admission row; and a
change in liveness would decide what a node admits, which nothing in Phase 2 needs.

**D2 — Liveness is tracked once concluded.** The four new commands are admitted in `Bootstrapped` and `Reconfiguring`
alone. Before the conclusion a node's peers are still starting, and the bootstrap's own signals and its deadline are
the liveness that matters; a node probing from `Pinging` would suspect every peer that has not started yet. Every
member is alive when the node concludes; a member admitted or added later is alive when it becomes one.
*Alternative:* from creation — refused for the reason given.

**D3 — The machine runs the detector; the caller runs the clock and the wire.** The roadmap's own rule: the machine
never probes, it says what to probe. At each `ProbeDue` the node names one member to probe: the one after the last
probed, in member order, wrapping around and skipping itself — SWIM's round-robin selection, which probes every member
once every N−1 periods. The ladder's property tier can check that as an invariant. Suspected members are still probed:
that is how they are revived. The period, the probe's timeout and the indirect probes' are the caller's, as the
deadline's is today.
*Alternatives:* the caller runs its own detector and reports only "suspected" and "revived" — a smaller machine, but
every consumer would write the probing itself and the specification would cover none of it; a random target, as in
the SWIM paper — refused: nothing in `faction` is random (`P0-ADR-DeterministicReplayable`), and round-robin is SWIM's
own deterministic variant.

**D4 — Any sign of life counts.** `PeerHeard { peer_id }` reports that the caller heard from the member: an answer to
a probe, direct or relayed by a helper, or anything else the caller treats as proof of life. A consumer that already
exchanges traffic with its members can report it, and a probe it would have sent is then answered by what it already
heard. The protocol crate reports probe answers alone.
*Alternative:* probe answers only — refused: whether other traffic proves a member alive is the caller's to judge,
and refusing it would make such a caller probe members it hears from constantly.

**D5 — Two levels, alive and suspected; nothing is confirmed dead.** In SWIM a suspect left unrefuted for a timeout is
confirmed failed and dropped from the member list. In `faction` a member is dropped only by a removal, which the layer
above commits (Phase 4). So Phase 2 stops at suspicion: a suspected member stays suspected until it is heard from, and
stays a member. Phase 4's rules can then read the suspected members — a suspect's removal is valid where a live
member's would break quorum.
*Alternative:* a third level, `Down`, after a further timeout the caller reports — refused for now: nothing in
`faction` would act on it, and Phase 4 can add it if its removal rules need one.

**D6 — Each node judges for itself; nothing is gossiped.** SWIM spreads suspicions by piggybacking them on probes,
with incarnation numbers so that a suspected member can refute its suspicion. Phase 2's view is the node's own:
whatever it suspects, it failed to reach, both directly and through others. Agreement on who has gone is the
committing layer's, as a removal's will be. Incarnation numbers belong with epochs (Phase 5).
*Alternative:* piggybacked suspicion — refused: it needs incarnations to be refutable and a field on every message,
and it would make one node's view depend on another node's.

**D7 — Quorum: the members not suspected, counted against the required count in effect.** The node always counts
itself as alive. The node checks this again after every accepted command, in the concluded stages, that changes the
members, the suspected members or the count. That covers a suspicion, a revival, an admission, a commit and a new
count. Each crossing is reported once, at the end of the command's outcomes.
A node concludes with every member alive and at least the required count of them, so quorum is always held at the
conclusion.
*Alternative:* a majority of the members — refused: `faction` never derives a count from the number of peers (§2 of
the specification, `P0-ADR-ProtocolAgnostic`).

**D8 — Indirect probes: the machine names the candidates, the caller decides how many to ask.**
`SendIndirectProbes { peer_id, via }` lists every alive member but the node and the missed one. The list starts
after the missed member in member order and wraps, so successive probes ask different helpers. The caller asks
the first k, k being its own choice, like its timeouts. The configuration does not change.
*Alternative:* k in the configuration, the machine naming exactly k — refused: no row depends on k, only the
protocol's sending does, and the caller already chooses how its probes travel.

**D9 — Names in the specification's vocabulary.** The roadmap's drafts are folded into four commands and eleven outcomes.
`IndirectProbeAck { target, via }` and `Revive { node }` both become `PeerHeard`: the core does not care who relayed
the answer, and hearing from a member is what revives it. `EmitQuorumLost` becomes `QuorumLost`, which says what
happened.

**D10 — Every 0.2.0 vector is migrated, its meaning unchanged.** As Phase 3's D7: the view grows by the suspected
members, the missed rounds, the last member probed and the quorum — none, none, none, and `NotTracked` before the
conclusion or `Held` after it, in every 0.2.0 step — the concluded stages' admissible lists by the four new commands,
and every configuration by a suspicion threshold of 1 (D12). All 62 vectors change mechanically in those places,
and the model checks them before the crate runs them.

**D11 — The system tests silence nodes and cut links.** The scenarios need a member that stops answering and a link
that fails while the rest work. The harness gains both, through the `FaultyTransport` decorator `OPEN_POINTS.md`
deferred to "the Phase-2 wiring", wired into `ClusterBuilder` with its fault logging. Silencing is done in the
transport, so it reads the same in all ten spawn and transport cases. The delivery-quality assessment of that open
point — loss, duplication, reordering against the bootstrap's retries — stays a piece of its own: nothing in Phase 2
depends on its verdict.
*Alternative:* kill processes — refused: it only works for the `Process` spawn, and a killed node cannot be revived.

**D12 — A suspicion threshold, given to the node when it is created.** Some deployments lose a period's traffic and
recover the next — a lossy link, a small board too busy to answer for a while — and some never do. The configuration
gains a suspicion threshold: how many periods running a member must miss, directly and through every helper asked,
before it is suspected. It is at least 1, and a configuration with 0 is outside the specification. Each member has a
count of missed periods. A period that ends with the indirect probes missed adds one; any sign of life puts it back
to none. The member is suspected when its count reaches the threshold. A threshold of 1 suspects at the first missed
period. The threshold is each node's own and never changes, like the rest of the configuration
(`P1-ADR-ConfigIsImmutableGenesis`); nodes of one cluster may hold different ones, since each judges for itself (D6).
A newcomer is given its threshold with its id, by its own deployment; the membership it is sent does not carry one.
The view shows each member's count, so a vector can check it.
How tolerant a node is, is then set in three places, all its deployment's:
- the period and the timeouts tolerate slow answers;
- the helpers asked tolerate one broken link;
- the threshold tolerates a burst of loss.

*Alternatives:* a command that sets the threshold, as `RequiredCountChanged` sets the count — refused: it would still
need a value to start from, and nothing asks to change it while a node runs; the caller counting the missed periods
itself and reporting the miss only at its own threshold — refused: every consumer would write the counting, and the
specification would cover none of it.

## 4. The specification, 0.3.0

**Configuration:** the node's peer id, its genesis peers, its required count and, new, its suspicion threshold.

**Commands**, in the commands' order, before `Probe`:

| command | carries | admitted in |
|---|---|---|
| `ProbeDue` | — | `Bootstrapped`, `Reconfiguring` |
| `PeerHeard` | `peer_id` | `Bootstrapped`, `Reconfiguring` |
| `ProbeMissed` | `peer_id` | `Bootstrapped`, `Reconfiguring` |
| `IndirectProbesMissed` | `peer_id` | `Bootstrapped`, `Reconfiguring` |

**Outcomes:**
- `SendProbe { peer_id }`: probe the member.
- `NothingToProbe`: the node is its only member.
- `SendIndirectProbes { peer_id, via }`: ask the members listed to probe `peer_id`.
- `MissCounted { peer_id, misses }`: a missed period counted, below the threshold.
- `PeerSuspected { peer_id }`, `PeerRevived { peer_id }`.
- `AliveNoted { peer_id }`: an alive member heard from; its count of missed periods goes back to none.
- `AlreadySuspected { peer_id }`: a miss of a member already suspected; nothing changes.
- `SelfIgnored`: a command that names the node itself.
- `QuorumLost`, `QuorumRestored`.
- Reused: `NonMemberIgnored`.

**Rows**: 0.2.0's twenty-nine keep their order and meaning. New rows, in the order they are checked:

| row | stages | command | when | result | change |
|---|---|---|---|---|---|
| `probe-alone` | concluded | `ProbeDue` | the node is its only member | `NothingToProbe` | — |
| `probe-due` | concluded | `ProbeDue` | | `SendProbe` | the peer is the last probed |
| `liveness-self` | concluded | `PeerHeard`, `ProbeMissed`, `IndirectProbesMissed` | the peer is the node itself | `SelfIgnored` | — |
| `liveness-non-member` | concluded | the same three | the peer is not a member | `NonMemberIgnored` | — |
| `heard-revived` | concluded | `PeerHeard` | the peer is suspected | `PeerRevived` [`QuorumRestored`] | no longer suspected |
| `heard-alive` | concluded | `PeerHeard` | | `AliveNoted` | its missed periods back to none |
| `miss-suspected` | concluded | `ProbeMissed`, `IndirectProbesMissed` | the peer is suspected | `AlreadySuspected` | — |
| `probe-missed` | concluded | `ProbeMissed` | | `SendIndirectProbes` | — |
| `peer-suspected` | concluded | `IndirectProbesMissed` | this miss brings its count to the threshold | `PeerSuspected` [`QuorumLost`] | the peer is the last suspected; its count cleared |
| `miss-counted` | concluded | `IndirectProbesMissed` | | `MissCounted` | its count grows by one |

Each existing row that changes the members or the count in a concluded stage — `JoinApproved`'s admission,
`AdditionCommitted`'s two, `RequiredCountChanged`'s — gains a twin before it. The twin applies when the command
crosses the quorum, and appends `QuorumRestored` or `QuorumLost` to the row's own outcomes.

**Prose:**
- §2: the configuration's suspicion threshold; a node also holds the suspected members, each member's missed
  periods and the last member probed.
- §4: the view's four new fields.
- §5: admission.
- A new §13, *Liveness*, and §14, *Quorum*.
- §1's list of commands.

**Schema:** the configuration's threshold, the commands, the outcomes and the view's fields.

**Vectors:** at least one per new row, the migration of D10, the round-robin wrapping past a member added
mid-round, and a threshold above 1: a member missing one period fewer, heard from, then missing the threshold's
number of periods.

## 5. The crate, 0.7.0

`Config::new` takes the suspicion threshold. The suspected members, each member's missed periods and the last
member probed are part of the concluded states' data. `Bootstrapped` and
`Reconfiguring` share one liveness struct, so the rows are written once (`P2-ADR-StateStructsBehindStageEnum`).
`Command`, `Outcome` and `ClusterView` grow, which breaks the API, hence 0.7.0. The ladder's tiers follow the matrix:
- the matrix tier: the new commands in every stage;
- the property tier, against the model, with three invariants: between two changes of the members, every member is
  probed within N−1 consecutive `ProbeDue`s; a member is suspected only after missing the threshold's number of
  periods running, with no sign of life among them; and quorum is held exactly when the members not suspected reach
  the count;
- the replay tier: unchanged in shape.

Both coverage gates stay at 100%: every new line must be reached by the vectors alone.

## 6. The protocol

`faction-protocol` turns the outcomes into messages and timers.

**Four messages on the wire.**
- `Probe { from, relay_for }`: a probe, `relay_for` naming the node a helper probes on behalf of.
- `ProbeAnswer { from, relay_for }`: the answer, sent to whoever probed.
- `IndirectProbe { from, target }`: a node asks a helper to probe `target`.
- `IndirectProbeAnswer { from, target }`: a helper relays `target`'s answer to the node that asked.

A helper keeps no table of the probes it relays: the probe and its answer carry whom they are for.

**Timers** (the caller decides each delay, as the deadline's today):
- `ProbeDue`: the period. It is scheduled when the node concludes and rescheduled at each `ProbeDue`.
- `ProbeMissed { peer_id }`: the probe's timeout.
- `IndirectProbesMissed { peer_id }`: the rest of the period.

**Translation.**
- `SendProbe { p }` becomes `Send { to: p, Probe }` and schedules `ProbeMissed { p }`.
- `SendIndirectProbes { p, via }` sends `IndirectProbe { target: p }` to the first k of `via`, k being the protocol's
  parameter. It schedules `IndirectProbesMissed { p }`.
- A `ProbeAnswer` is `PeerHeard { from }`, and an `IndirectProbeAnswer` is `PeerHeard { target }`: the helper only
  relayed it. Either cancels both of the member's timers.
- A received `Probe` is answered with `ProbeAnswer`. The prober is alive too, so it is also `PeerHeard`.
- A received `IndirectProbe` sends `Probe { relay_for }` to the target, and a `ProbeAnswer` that carries a
  `relay_for` is forwarded as an `IndirectProbeAnswer`.
- `PeerSuspected`, `PeerRevived`, `QuorumLost` and `QuorumRestored` send nothing. The caller reads them from the
  result, and the system tests from the snapshot.

**The newcomer** is given its suspicion threshold with its id and observer, and creates its node with it once the
membership arrives (D12).

**Tests** (`crates/protocol/tests/`, written first): each translation; the relaying, in both directions; the period
being rescheduled; no `ProbeDue` before the conclusion. Coverage stays at 100%.

## 7. The system tests

**`validation/protocol-validation`**, the deterministic in-process cluster, carries what a real transport cannot force:
1. an answer lost on the direct path, saved by an indirect one;
2. a member whose answers are all lost is suspected; delivered again, it is revived;
3. every member is probed within one round;
4. a member added mid-round is probed in the same round;
5. a raised count loses quorum, a lowered one restores it;
6. a node still bootstrapping probes nothing;
7. at a threshold of 3, a member missing two periods, then heard from, then missing two more is never suspected, and
   missing a third is.

**`validation/system-tests`** runs each scenario in all ten spawn and transport cases:

| scenario | asserts |
|---|---|
| `a_silenced_member_is_suspected_by_every_other` | five members, count three; one silenced; every other node suspects it, quorum held |
| `a_member_heard_again_is_revived` | the same member unsilenced; no node suspects anyone |
| `silencing_too_many_loses_quorum_and_their_return_restores_it` | three of five silenced; the two left report quorum lost; unsilenced, held again |
| `a_member_cut_off_from_one_node_is_saved_by_indirect_probes` | one link cut; no node suspects either end |
| `an_added_member_is_probed_once_committed` | after an addition, the newcomer silenced is suspected by every other |

That is fifty new system tests.

**The harness.**
- `Cluster` gains `silence(node)`, `unsilence(node)` and `cut(a, b)`, through `FaultyTransport` (D11).
- `ClusterBuilder` takes the suspicion threshold its nodes are created with. The scenarios run at 2, so each
  suspicion passes through a counted miss first; the process's command line carries it.
- `NodeSnapshot` gains the suspected members and the quorum. The process's `state` line gains both.
- The wire's shared codec gains the four messages, and `FaultyTransport`'s `MessageKind` gains four kinds.
- The probe period and the timeouts come from `TimerDelay`, as the retries' delays do.

**Order:** protocol tests, then `protocol-validation` scenarios, then the codec tests, then the system scenarios.
Each is seen failing before what it covers is written.

## 8. The independent model

`crates/spec-model/` follows the specification before the crate does:
- the four commands and the outcomes, in `vocabulary/`;
- each new row's situation and change, in the rule book;
- the suspected members and the last member probed, in its node state.

The 0.3.0 vectors, new and migrated, pass against the model before the crate is touched.

## 9. Order of work

1. The specification 0.3.0: prose, schema, matrix, the new vectors and the migration.
2. The model, until every vector passes against it.
3. The crate, test first, until the conformance tier passes; then the matrix and property tiers.
4. The protocol, then `protocol-validation`, then the harness and the system tests (§6, §7).
5. The docs:
   - a Phase 2 ADR: liveness reported in the view, judged by each node, never confirming a member dead;
   - ROADMAP: Phase 2's names as in §4, and Phase 4's rejection reason `NodeAlreadySuspected` flagged as clashing
     with its own table;
   - ARCHITECTURE: L2 removed;
   - CHANGELOG and README;
   - OPEN_POINTS: the fault wiring moves out, the assessment stays.
6. Release 0.7.0, at the owner's word.

## 10. Out of scope

- Removal (Phase 4).
- Epochs, incarnations and refuting a suspicion (Phase 5).
- Gossip.
- Confirming a suspect dead.
- Lifeguard's adjustments to the local timeouts.
- Choosing any period, timeout or k.
- The delivery-quality assessment.
- Deciding anything from a suspicion.
