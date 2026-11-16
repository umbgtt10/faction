# [PROPOSAL] Phase 5 — epochs and rejoin

**Status:** accepted 2026-11-16, every decision, D1 to D10, as proposed: the owner's "Agreed!" to §12's four points
-- the commit's epoch, claims refused and liveness never, no new stages, a membership not naming a node removing it.
Built on the branch `phase-5`.

**Releases:** specification 0.5.0, then the crate 0.9.0.
**Order:** after Phase 4; before Phase 6, which needs the epoch to order the changes it queues.

`faction` answers: *"Can the cluster survive membership changes across restarts, and keep a stale node from counting?"*

---

## 1. What Phase 5 adds

Faction knows the members a node holds, and nothing says which membership is newer than which. Three gaps follow,
the roadmap's L5:

- **A restarted node starts from genesis.** It is a fresh machine from its configuration (`P1-ADR-ConfigIsImmutableGenesis`);
  replaying its input log is the caller's (`P1-ADR-StatefulPersistencyFree`). A node not replayed holds the genesis
  members after the cluster has added and removed some, and nothing tells it.
- **A removed node has no way back.** `Removed` admits a probe alone, "until Phase 5 gives a way back"
  (`P1-ADR-TerminalStatesAreNotSinks`). Its id can be added again, as any non-member's, but the node itself cannot
  take a membership: a running protocol acknowledges a `Membership` message and never applies it.
- **A stale node still counts.** A node that missed a change -- restarted, partitioned, or simply slower to apply a
  commit -- signals participation and readiness, and they count toward the quorum of a membership it does not hold.

Phase 5 adds one number and three uses of it:

1. **The epoch:** the version of the membership a node holds, set by each change the layer above commits.
2. **Staleness:** a claim from a node behind -- participation, readiness, an addition's acknowledgement -- is refused,
   by a defined outcome, and the node behind is sent the membership.
3. **Adoption:** a node handed a membership at a higher epoch takes it, whatever its stage. That one move is a
   restarted node's return, a partitioned node's after the partition heals, and a removed node's way back.

And it adds them **additively**, the first phase under the specification's growth rule (§1): every vector released
so far passes unchanged.

## 2. Where it starts

From a reading of the code at `0d97ff1`, each fact cited there:

- **No version anywhere.** Nothing counts committed changes; no field of `Config`, `ClusterView`, `ProcessResult` or
  any `TransportMessage` carries a version, an epoch or a sequence. Earlier freshness machinery was removed in 0.3.0
  and 0.3.1.
- **Commits come from the layer above**, to every node, in any stage but `Removed`: `AdditionCommitted`,
  `RemovalCommitted` and `RequiredCountChanged`. Faction proposes (`ProposeAddition`, `ProposeRemoval`) and never
  commits.
- **Joins are local.** `JoinRequested`, `JoinApproved` and `JoinRejected` are the caller's decisions on each node,
  carried by no message; an approved peer is appended to the members.
- **"Rejoin" today is `AcknowledgeRejoin`:** a concluded or collecting node answers a member's participation with its
  readiness. It is how a restarted node rejoins *a membership that has not changed*.
- **A newcomer takes a membership** through the protocol's `Newcomer`: it builds a fresh configuration from the
  `Membership` message -- the members and the required count -- and a fresh machine.
- **Nothing tests** a restart, a removed node re-added, or a stale membership.

## 3. The flow

**A restart.** Node 3 restarts after the cluster added node 6 (epoch 1). It is fresh from genesis, at epoch 0, and
pings. Node 1 meets a participation at epoch 0, below its own: it refuses it as stale, and sends node 3 the
membership at epoch 1. Node 3 adopts it -- members, count, epoch -- and bootstraps on, its signals now at epoch 1;
node 1 answers them with its readiness, as `AcknowledgeRejoin` always has.

**A removed node, re-added.** Node 4 was removed at epoch 2; it is `Removed`. The layer above decides to take it
back: a member coordinates its addition as Phase 3's, `SendMembership` to node 4. Node 4 -- now admitting a
membership at a higher epoch that names it -- adopts it and leaves `Removed` for `Initial`, a fresh node at that
epoch. It acknowledges; the addition is proposed and committed at epoch 3; node 4 bootstraps among the members. Any
message node 4 sent before its removal carries epoch 2 or less, and is stale everywhere.

**A partition heals.** Node 5 was cut off while a removal committed at epoch 4. When the link mends, its readiness
reaches node 1 at epoch 3: stale, refused, and node 5 is sent the membership at epoch 4. It adopts it. Its probe
answers counted throughout: being alive is not a claim on a membership.

## 4. Decisions

**D1. The commit carries the epoch; faction never counts.** `AdditionCommitted`, `RemovalCommitted` and
`RequiredCountChanged` each gain an `epoch`: the layer above's own number for the change, strictly increasing --
Raft's log index, IBFT's height. A node takes the commit's epoch as its own. *Why not count:* commits reach nodes at
different moments, and a node that adopted a membership early would count a commit twice; with the commit's own
epoch, a commit at or below the node's epoch is one it already holds, `CommitIgnored`, and applying is idempotent.
*Rejected:* faction counting applied changes -- not idempotent across adoption, and two layers' numbers to reconcile.

**D2. Joins do not move the epoch.** `JoinApproved` is a local admission before or around bootstrapping, decided
on each node by the caller, carried by no message (Phase 1). The epoch versions what the layer above commits.

**D3. A node's own signals carry its epoch; claims from behind are refused, liveness never.**
`ParticipationObserved`, `ReadyObserved` and `MembershipAcknowledged` gain the sender's `epoch`. Below the node's own,
the claim is refused -- `StaleEpochIgnored { peer_id, epoch }` -- and the node sends the sender the membership
(`SendMembership`, Phase 3's outcome, now carrying the epoch). Above the node's own, the claim is taken as before: the
node behind is this one, and the membership will reach it. `PeerHeard` and the missed probes carry no epoch: a slow
commit must never get a live member suspected, and removed. *Why only claims:* the split-brain the roadmap names is
a stale node counting toward a quorum; refusing liveness would turn every commit's propagation into suspicion.

**D4. Adoption: a membership at a higher epoch is taken in every stage.** A new command, `MembershipReceived
{ members, required_count, epoch }`, admitted in every stage. At or below the node's epoch it is
`MembershipIgnored`. Above it:
- naming the node, it is adopted -- `MembershipAdopted { epoch }`: the members, the count and the epoch replaced;
  a change in flight from an older epoch is abandoned, `ChangeAbandoned`, since the membership moved past it; the
  stage kept, its lists filtered to the new members, the quorum re-checked as a commit re-checks it;
- not naming it, the node learns it is not a member: from `Removed` it stays `Removed`; elsewhere it is removed, as
  its own `RemovalCommitted` would remove it, and knows the epoch.

**D5. The way back from `Removed` is adoption, into `Initial`.** A removed node adopting a membership that names it
leaves `Removed` for `Initial`: a fresh node at that epoch, which bootstraps among the members as a newcomer does.
`P1-ADR-TerminalStatesAreNotSinks` is amended: `Removed` is left by adoption alone.

**D6. No new stages: the roadmap's `SplitSuspected` and `Rejoining(PeerId)` are dropped.**
- A node behind is not a stage: it learns it is behind only by being sent the membership, and adopts it in the same
  call. Liveness is reported, never staged (`P1-ADR-LivenessReportedNotStaged`); staleness is the same.
- Rejoining is `Initial` and bootstrapping again, at the adopted epoch: the stages a newcomer passes through.
- *Kept from the roadmap:* the epoch, stale-epoch rejection with a defined outcome, a removed node's way back, and a
  stale node's claim on the quorum refused.
- *Dropped with them:* "the epoch at which each node last confirmed membership". Each signal carries its sender's
  epoch, which says the same thing per message.

**D7. The view and the configuration carry the epoch.** `ClusterView` gains `epoch`. `Config` gains an optional
starting epoch with its members and count: a node created from a snapshot of the membership -- a restart that kept
one, IBFT's in its hard state -- starts at that epoch, its own genesis (`P1-ADR-ConfigIsImmutableGenesis`
unchanged: a configuration is still a node's genesis, now at an epoch).

**D8. Growth, exactly.** Epoch 0 is every member's initial value: omitted from the configuration, the view, the
signals and the commits wherever it is 0, and a commit or a signal with no epoch is at epoch 0. Every vector
released through 0.4.1 holds epoch 0 throughout and answers alike. The new command and outcomes are named in no
earlier vector's admissible commands. **Every released vector passes unchanged; `released.json` is untouched.**

**D9. Faction adopts what the caller hands it.** Whether a membership is genuine -- signed by a quorum, from a
member -- is the caller's to check before it reports it, as a vote's signature is cartel's caller's. A liar claiming
a high epoch is outside faction, as the roadmap's lying peer is; the Notary bookkeeper is where signed memberships
belong. Faction's own protocol, its reference runtime, trusts its transport, as it does today.

**D10. Names:** the command `MembershipReceived`; the outcomes `StaleEpochIgnored`, `CommitIgnored`,
`MembershipIgnored`, `MembershipAdopted`, `ChangeAbandoned`; the member `epoch` everywhere. Each follows the
vocabulary: commands say what the caller observed, outcomes what happened.

## 5. The specification, 0.5.0

- **§1:** the epoch, and D8's reading of an absent epoch.
- **§2 and the schema:** the configuration's and the view's optional `epoch`.
- **The commands:** the epoch on the three signals and the three commits; `MembershipReceived`.
- **The rows, about twelve new:**
  - a signal from behind;
  - a commit already held;
  - a membership at or below the epoch;
  - a membership adopted, in each stage that keeps its stage, and with a change in flight;
  - a membership not naming the node, from `Removed` and from elsewhere;
  - `Removed` left for `Initial`.
- **Every existing row is unchanged at epoch 0.**
- **Vectors 94 onwards, written by hand:** each new row, and the three flows of §3 in one node's view.
  `released.json` gains them at 0.5.0.

## 6. The crate, 0.9.0

A minor release: new fields on `Config`, `ClusterView` and the commands, a new command and new outcomes. Every
existing call compiles once its commands name an epoch or take a default, and answers alike at epoch 0. The crate's
`Stage` and its states gain no variant.

## 7. The protocol

- **`TransportMessage`:** `Ping`, `Ready`, `Membership` and `MembershipAcknowledged` carry the sender's epoch.
  Probes and their answers do not (D3).
- **The translator:** `StaleEpochIgnored` sends the stale peer a `Membership`. A `Membership` at a higher epoch
  reaches a running node as `MembershipReceived`, a `Removed` one included; today only a `Newcomer` takes one.
- **The retry gap the reading found:** `retry` stops only once `Bootstrapped`, so a node removed before concluding
  keeps broadcasting. It now stops at `Removed` too, with a test.

## 8. The system tests

Five new scenarios, each across the ten spawn-and-transport cases, as Phase 4's:

1. a restarted node catches up after an addition, from genesis;
2. a restarted node created from a membership snapshot starts at its epoch;
3. a removed node is re-added and rejoins;
4. a partitioned node misses a removal and catches up once the link mends, never suspected for being behind;
5. a stale node's readiness is refused until it adopts the membership.

The protocol-validation tier takes the same five in-process, and the harness gains a restart: a node dropped and
created again, from genesis or from a snapshot.

## 9. The independent model

The epoch in the node's state; situations for a sender behind, a commit already held, and a membership above the
epoch that names the node or does not; changes for adoption and the way out of `Removed`. Written from the prose and
the table, never from the crate.

## 10. Order of work

1. This proposal, agreed.
2. The specification 0.5.0, its vectors and the model, the model answering every vector before the crate runs one.
3. The crate and the ladder: unit tier, conformance, the matrix, properties.
   - New property: a node's epoch never decreases.
   - New property: a claim from behind never counts.
4. The protocol and protocol-validation.
5. The harness's restart and the system tests.
6. The docs: an ADR for the epoch (`P1-ADR-TheEpochComesWithTheCommit`), the amended `TerminalStatesAreNotSinks`,
   the README, ARCHITECTURE, ROADMAP and CHANGELOG.
7. The release.

Each step on this branch, every gate green on every commit.

## 11. Out of scope

- **Concurrent changes** -- Phase 6, which will order its queue by epoch.
- **Verifying a membership** (D9) -- the caller's, and the Notary bookkeeper's.
- **Numbered results** (`OPEN_POINTS.md`) -- a different number: a result's place in a node's log, where the epoch is
  the membership's version.
- **`PeerId` made generic** -- after Phase 6.
- **Raft's and IBFT's adoption of 0.9.0** -- their own steps: Raft passes the commit's log index as the epoch, IBFT
  its height, and IBFT's restarted node stops acknowledging its rejoin by hand.

## 12. For the owner to decide

1. **D1 or counting.** The proposal has the layer above's number as the epoch.
2. **D3's line.** Claims refused from behind, liveness never.
3. **D6.** No `SplitSuspected` or `Rejoining` stage, departing from the roadmap's draft.
4. **D4 for a node the membership does not name.** Outside `Removed`, it removes itself, as its own commit would.
