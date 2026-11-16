# faction — the specification

**Version 0.5.1.** The four parts of this specification are versioned together: this prose, the data model in
[`schema.json`](schema.json), the decision table in [`matrix.json`](matrix.json) and the conformance vectors in
[`vectors/`](vectors/). It is normative; the `faction` crate is one implementation of it, and where the two disagree
the specification is right ([`P0-ADR-SpecificationIsNormativeCrateIsReference`](../docs/ADRs/P0-ADR-SpecificationIsNormativeCrateIsReference.md)).

**What 0.5.0 covers:** faction whole as it stands after its first phase, its addition, its failure detection, its
removal and its epochs — a
node created from its configuration, and the observer it reports every call to; bootstrapping, from the first signal to
the conclusion, with the deadline; joining, a newcomer admitted or denied in every stage; addition, a concluded node
growing by one through the layer that commits changes above it; removal, a concluded cluster shrinking by one through
the same layer, and a node removed from its own; the required count in effect; liveness, a concluded
node probing its members and suspecting those it cannot reach; quorum, its members not suspected against the count;
the epoch, the version of the membership a node holds, with which a claim from behind is refused, a commit already
held ignored and a newer membership adopted, a removed node's way back among them; and the probe. Each command with
every row it has. 0.5.0 is the first version to grow by the rule of §1: every vector 0.4.0 and 0.4.1 released is
answered alike. 0.5.1 answers every command as 0.5.0 does, and states what the layer above owes a node: its commits in
epoch order (§16).

## 1. Conventions

**Every number is a string of decimal digits**, with no sign, no leading zero and no exponent, its value within
`0..=18446744073709551615`. Many JSON parsers read numbers as 64-bit floats, exact only to 2⁵³, and a peer id is
any u64.

**Every value of an enumeration is an object with exactly one member**, named for the value:
`{"ParticipationObserved": {"peer_id": "2"}}`, `{"Concluded": {"mode": {"Bootstrapped": {}}}}`. A value that carries
nothing carries the empty object: `{"Probe": {}}`, `{"Fresh": {}}`.

**Names are the vocabulary's.** A command says what the caller observed — a peer's signal, its own completion, the
deadline, a join decision, an addition's or a removal's progress, a new required count, a membership handed to it, a
probe period, a member heard from or missed — or, `Probe`, asks. An outcome says what happened to the signal, or, in the imperative, what the caller must now do:
`BroadcastLocalReady`, `AcknowledgeRejoin`, `EmitJoinRequest`, `SendMembership`, `ProposeAddition`, `ProposeRemoval`,
`SendProbe`, `SendIndirectProbes`.

**A list of commands by name** is in the commands' order: `ParticipationObserved`, `ReadyObserved`,
`LocalParticipationCompleted`, `DeadlineExpired`, `JoinRequested`, `JoinApproved`, `JoinRejected`, `AdditionRequested`,
`MembershipAcknowledged`, `AdditionCommitted`, `AdditionAborted`, `RemovalRequested`, `RemovalCommitted`,
`RemovalAborted`, `RequiredCountChanged`, `MembershipReceived`, `ProbeDue`, `PeerHeard`, `ProbeMissed`,
`IndirectProbesMissed`, `Probe`.

**A list of peers is in arrival order.** The members are the configuration's peers in its order — or, once the node
has adopted a membership, that membership's members in its order (§16) — then each admitted or added peer as it is
admitted or added, a removed peer gone from wherever it was; the pinging and the collecting peers are in the order their signals were accepted;
the suspected peers in the order they were suspected; and the members who have missed periods in the members' order. The same
configuration and the same commands give the same lists, so two implementations' answers to the same calls are
identical, and a list never holds a peer twice
([`P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder`](../docs/ADRs/P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder.md)).

**Outcomes are a sequence**: their order is part of the answer.

**Growth.** A member a later version adds to a form is omitted wherever it holds the value that keeps the earlier
version's behaviour, and a command a later version adds is named in no earlier vector's admissible commands: an
implementation conforms to a vector of an earlier version when the commands it admits, among those that version
names, are the vector's. The commands a released version names are those [`released.json`](released.json) lists at
that version or an earlier one (§17). Every answer this version gives is therefore given the same by every later
version, and every vector released here passes unchanged there. 0.5.0 grows by the rule: the epoch (§16), a member of
the configuration, the view, three signals and three commits, is omitted wherever it is 0, and a form with no epoch
holds epoch 0; the command `MembershipReceived` is new, and so are the outcomes it and the epoch bring. Before the rule was written the encoding grew in place: 0.2.0,
0.3.0 and 0.4.0 each added fields to the view and the configuration, and commands to what earlier stages admit,
rewriting the vectors released before them -- never an outcome -- and 0.4.0 renamed the view's `addition` to
`reconfiguration`.

## 2. A node

**A node is created from its configuration**, and with an observer (§3). The configuration is the node's own peer
id, its genesis peers, a required count, a suspicion threshold and an epoch, 0 unless it states another: the epoch
of the membership its peers and count are (§16). The genesis peers include the node itself and list no peer twice; a
configuration that breaks either is outside this specification. The required count is how many confirmations
conclude bootstrapping, and any count is one: it is never derived from the number of peers
([`P0-ADR-ProtocolAgnostic`](../docs/ADRs/P0-ADR-ProtocolAgnostic.md)). The suspicion threshold is how many periods running a member
must miss before the node suspects it (§14); it is at least one, and a configuration with none is outside this
specification. The configuration never changes
([`P1-ADR-ConfigIsImmutableGenesis`](../docs/ADRs/P1-ADR-ConfigIsImmutableGenesis.md)); the count in effect starts
as the configuration's, and changes by its own command alone (§13).

**A node is in one of six stages:** `Initial`, created and given nothing yet; `Pinging`, gathering its peers'
participation; `Collecting`, its own participation completed, gathering readiness; `Bootstrapped`, concluded; and
`Reconfiguring`, concluded with a change in flight, an addition (§11) or a removal (§12); and `Removed`, removed from
its own cluster (§12).
A node is created `Initial`, leaves it on the first accepted command that is not a membership decision — a join
decision, a committed addition or removal, a new required count, or a membership handed to it — or on its own removal,
and returns to it from `Removed` alone, adopting a membership that names it (§16).
`Bootstrapped` is the only conclusion: a node that has concluded is `Bootstrapped`, or `Reconfiguring` while a change
is in flight, until it is removed, and no command takes it back to an earlier stage. A removed node stays `Removed`
until it adopts a membership that names it.

**Besides its stage a node holds** its members, its pinging peers, its collecting peers, whether it has missed its
deadline, the required count in effect, the change in flight, its suspected peers, the periods each member has missed,
the member it probed last and its epoch. A fresh node's members are its genesis peers; it has no pinging or collecting
peer, has not missed its deadline, its required count is the configuration's, it has no change in flight, suspects no
one, counts no missed period, has probed no one, and its epoch is the configuration's. Members are added (§10, §11),
removed (§12) and replaced with a membership the node adopts (§16).

## 3. One entry point, one result

**Every call is a command, and every command is answered with a result**: `Accepted`, carrying the outcomes, the
view and the admissible commands; `Rejected`, carrying the view and the admissible commands; or `Probed`, carrying
the same two ([`P1-ADR-SingleEntryPoint`](../docs/ADRs/P1-ADR-SingleEntryPoint.md)). The view and the admissible
commands a result carries are the node's after the call.

**The answer is total and deterministic.** Every command has exactly one result in every stage, given by the first
row of [`matrix.json`](matrix.json) that applies, in the matrix's order. The same state and the same command give the
same result ([`P1-ADR-TotalityExhaustiveMatrix`](../docs/ADRs/P1-ADR-TotalityExhaustiveMatrix.md),
[`P0-ADR-DeterministicReplayable`](../docs/ADRs/P0-ADR-DeterministicReplayable.md)).

**`Probe` is always answered `Probed`, and changes nothing.** It is admitted in every stage.

**A command the stage does not admit is answered `Rejected`, and changes nothing.** What each stage admits is §5.
The admissible commands a result lists are exactly those its stage admits, `Probe` among them.

**An accepted command is answered `Accepted`**, with the outcomes and the change its row gives (§6–§16). An outcome
that names a peer names the command's `peer_id`, but `SendProbe`, which names the member to probe (§14), and
`MemberRemoved` answering a membership, which names the node itself (§16); an outcome that carries a required count
carries the command's, and one that carries an epoch the command's.

**Every call is reported to the node's observer**, given when the node is created: the command and its result,
exactly once, by the time the result is returned — a rejected command and a probe like the rest. The observer is
told and answers nothing, so it cannot change a result; a node with no use for one is given an observer that does
nothing ([`P2-ADR-TotalObservability`](../docs/ADRs/P2-ADR-TotalObservability.md)).

**A node's log is its configuration, then one step per call** — the command and its result — in the order the calls
were made, as its observer was told them. Created from the configuration and given each step's command in turn, a
conforming node answers each step's result: a log that does not replay was cut, reordered or altered, or written by
a node that does not conform.

## 4. The view

**The view is what a node shows of its state**: its peer state, whether its pinging is completed, its pinging
peers, its collecting peers, its required count, whether it has missed its deadline, its members, its reconfiguration, its
suspected peers, its missed periods, the member it probed last, its quorum, and its epoch (§16), omitted at 0.

**The peer state** is `Fresh` while the node is `Initial`, `Bootstrapped` once it has concluded — `Bootstrapped` or
`Reconfiguring` — `Removed` once it is removed, and otherwise its stage, `Pinging` or `Collecting`, or `TimedOut` if it has missed its deadline. A missed deadline is a fact the view
reports, not a stage: a `TimedOut` node goes on as its stage does, and may still conclude, after which its peer
state is `Bootstrapped` and its view still says the deadline was missed
([`P1-ADR-TerminalStatesAreNotSinks`](../docs/ADRs/P1-ADR-TerminalStatesAreNotSinks.md)).

**Pinging is completed** once the node's own participation is: in `Collecting`, `Bootstrapped` and `Reconfiguring`,
not before; in `Removed`, as it was when the node was removed.

**The required count** is the count in effect (§13). A node is concluded when its peer state is `Bootstrapped`.

**The reconfiguration** is `None` with no change in flight; `AdditionRequested`, naming the peer, once its addition is
requested, and `AdditionAcknowledged`, naming it, once the peer has acknowledged the membership (§11); and
`RemovalRequested`, naming the peer, once its removal is requested (§12).

**The suspected peers** are the members the node suspects (§14). **The missed periods** list each member that has
missed one or more periods running and is not suspected, with how many. **The member probed last** is `None` until the
node first probes, then `Probed`, naming the member.

**The quorum** is `NotTracked` until the node has concluded, and once it is removed; while it is concluded, `Held` while its members not suspected number
the required count or more, and `Lost` while they number fewer (§15).

## 5. Admission

Each stage admits these commands, and rejects the rest:

| Stage | Admits |
|---|---|
| `Initial` | every command but `DeadlineExpired`, `AdditionRequested`, `MembershipAcknowledged`, `AdditionAborted`, `RemovalRequested`, `RemovalAborted` and the four of liveness |
| `Pinging` | every command but `AdditionRequested`, `MembershipAcknowledged`, `AdditionAborted`, `RemovalRequested`, `RemovalAborted` and the four of liveness |
| `Collecting` | every command but `LocalParticipationCompleted`, `AdditionRequested`, `MembershipAcknowledged`, `AdditionAborted`, `RemovalRequested`, `RemovalAborted` and the four of liveness |
| `Bootstrapped` | `ParticipationObserved`, `JoinRequested`, `JoinApproved`, `JoinRejected`, `AdditionRequested`, `AdditionCommitted`, `RemovalRequested`, `RemovalCommitted`, `RequiredCountChanged`, `MembershipReceived`, `ProbeDue`, `PeerHeard`, `ProbeMissed`, `IndirectProbesMissed` and `Probe` |
| `Reconfiguring` | `ParticipationObserved`, `JoinRequested`, `JoinRejected`, `MembershipAcknowledged`, `AdditionCommitted`, `AdditionAborted`, `RemovalCommitted`, `RemovalAborted`, `RequiredCountChanged`, `MembershipReceived`, `ProbeDue`, `PeerHeard`, `ProbeMissed`, `IndirectProbesMissed` and `Probe` |
| `Removed` | `MembershipReceived` and `Probe` |

The four of liveness are `ProbeDue`, `PeerHeard`, `ProbeMissed` and `IndirectProbesMissed`. A node completes its own
participation once; a concluded node answers only a peer still trying to bootstrap, a newcomer, an addition, a removal, a
new count, a membership and liveness; an addition or a removal is requested of a concluded node alone, one change at a
time; a commit and a new count reach every node, whatever its stage, until it is removed, and a membership reaches every
node, whatever its stage; a node tracks its members' liveness once it has concluded, never before; and a removed node
answers a membership and a probe alone.

**Admission is decided when a command arrives, never by what a caller can construct.** Every command can be offered
to a node in every stage, and every vector runs on every conforming implementation: none relies on a type system to
keep a command from a stage
([`P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors`](../docs/ADRs/P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors.md)).

## 6. `ParticipationObserved`

**From a peer that is not a member**, the signal is ignored: `NonMemberIgnored`, and nothing changes, but that an
`Initial` node is now `Pinging`. A non-member's signal is not remembered: admitting the peer later does not count it.

**From a member behind** — its `epoch` below the node's — the claim is refused (§16).

**Before the node's own completion** — `Initial` or `Pinging` — a member's first participation is accepted,
`ParticipationAccepted`, and the peer becomes the last pinging peer; a repeat is ignored,
`DuplicateParticipationIgnored`. Either way the node is `Pinging`.

**After it** — `Collecting`, `Bootstrapped` or `Reconfiguring` — a member's participation means that peer is still
trying to bootstrap, and is answered `AcknowledgeRejoin`: the caller re-advertises the node's readiness to it. Nothing
changes. None of them is a sink ([`P1-ADR-CollectingIsNotASink`](../docs/ADRs/P1-ADR-CollectingIsNotASink.md),
[`P1-ADR-TerminalStatesAreNotSinks`](../docs/ADRs/P1-ADR-TerminalStatesAreNotSinks.md)).

## 7. `ReadyObserved`

**From a peer that is not a member**, the signal is ignored, as participation is (§6), and **from a member behind**
refused (§16). A concluded node does not admit readiness.

**A member's first readiness is accepted**, `ReadyAccepted`, and the peer becomes the last collecting peer; a repeat
is ignored, `DuplicateReadyIgnored`, and changes nothing.

**Before the node's own completion**, readiness only accumulates: an `Initial` or `Pinging` node is `Pinging` after
it, and nothing concludes — a fast peer's readiness waits, and counts once the node completes (§8).

**In `Collecting`**, readiness that brings the collecting peers to the required count concludes: `ReadyAccepted`,
then `Concluded` with the mode `Bootstrapped`, and the node is `Bootstrapped`.

## 8. `LocalParticipationCompleted`

**The node's own participation completes once**, from `Initial` or `Pinging`: `LocalParticipationCompleted`, then
`BroadcastLocalReady` — the caller announces the node's readiness to its peers. The node itself becomes the last
collecting peer, unless its own readiness was already accepted.

**If the collecting peers now number the required count or more**, the outcomes end with `Concluded`, mode
`Bootstrapped`, and the node is `Bootstrapped`; otherwise it is `Collecting`. Readiness accepted before completion
counts here.

## 9. `DeadlineExpired`

**In `Pinging` or `Collecting`, the deadline is missed**: `DeadlineMissed`, carrying how many collecting peers the
node has, and from then on the node has missed its deadline. Its stage does not change, and a later expiry is
answered alike: a missed deadline is not terminal.

## 10. Joining

**Join decisions never change the stage.** `JoinRequested` and `JoinRejected` are admitted in every stage but
`Removed`, and `JoinApproved` in every stage but `Reconfiguring` and `Removed`: no admission goes around a change in
flight.

**`JoinRequested`** is answered `EmitJoinRequest`: the caller forwards the newcomer's request to whoever decides.
Nothing changes; a repeated request is forwarded again.

**`JoinApproved`** admits the peer: `MemberAdmitted`, and the peer becomes the last member. A peer that is already a
member — the node itself among them — is answered `DuplicateMemberIgnored`, and nothing changes.

**`JoinRejected`** is answered `JoinDenied`, and nothing changes: a peer that is a member stays one.

**Admission changes who may be counted, never how many are required** (§13). On a concluded node, a member admitted
may restore its quorum (§15).

## 11. Addition

**A concluded node grows by one through the layer that commits changes above it**, one change in flight at a time.
The node that coordinates an addition is asked for it, hands the newcomer the membership, and tells the caller when to
propose it; every node applies it once it has committed. Who may be added, and how a change commits, are the caller's.

**`AdditionRequested`** starts an addition: `SendMembership` — the caller sends the newcomer the members, the
required count and the epoch the view shows — and the node is `Reconfiguring`, its reconfiguration `AdditionRequested`. A peer that is already a
member, the node itself among them, is answered `DuplicateMemberIgnored`, and nothing changes. `Reconfiguring` does not
admit a second request, of either kind.

**`MembershipAcknowledged`** from the peer whose addition is requested is answered `ProposeAddition` — the caller
proposes the addition to the layer above — and its reconfiguration is `AdditionAcknowledged`. From any other peer, once
acknowledged, or with a removal in flight, it is answered `AcknowledgementIgnored`, and nothing changes. From the peer
whose addition is requested, behind — its `epoch` below the node's — it is refused (§16).

**`AdditionAborted`** naming the peer of the addition in flight withdraws it: `AdditionWithdrawn`, and the node is
`Bootstrapped`, with no change in flight and the peer no member. Naming any other peer, or with a removal in flight,
it is answered `AbortIgnored`, and nothing changes.

**`AdditionCommitted`** makes its peer the last member: `MemberAdmitted`. The layer above reports a commit to every
node, whatever its stage, and every node applies it alike. On the coordinator, a commit naming the peer of the
addition in flight ends it, and the node is `Bootstrapped`; a commit naming another peer leaves the change in
flight. A peer that is already a member is answered `DuplicateMemberIgnored`, and nothing changes but the epoch. A
commit the node holds already is ignored, and every other moves its epoch (§16). The peer of the
addition in flight is never a member: its request is refused if it is one, and no admission is admitted meanwhile.

**An addition changes who may be counted, never how many are required** (§13). On a concluded node, a member added
may restore its quorum (§15).

## 12. Removal

**A concluded cluster shrinks by one through the layer that commits changes above it**, one change in flight at a time,
an addition or a removal
([`P1-ADR-RemovalThroughTheLayerThatCommits`](../docs/ADRs/P1-ADR-RemovalThroughTheLayerThatCommits.md)). The node that coordinates a removal is asked for it and tells the caller to propose it at
once — a member being removed has nothing to acknowledge, and may not answer at all; every node applies it once it has
committed. Who may be removed, and how a change commits, are the caller's: a suspected member stays a member until its
removal commits (§14).

**`RemovalRequested`** starts a removal: `ProposeRemoval` — the caller proposes the removal to the layer above — and the
node is `Reconfiguring`, its reconfiguration `RemovalRequested`. Any member may be named, the node itself among them. A
peer that is not a member is answered `NonMemberIgnored`, and nothing changes. `Reconfiguring` does not admit a second
request, of either kind.

**A removal that would lose the quorum is refused.** The removal of a member not suspected, when without it the members
not suspected would number fewer than the required count, is answered `RemovalRefused`, and nothing changes; the node
counts itself and is never suspected (§15). A suspect's removal is never refused for the quorum: it leaves the members
not suspected as they are. The node decides on its own view.

**`RemovalAborted`** naming the peer of the removal in flight withdraws it: `RemovalWithdrawn`, and the node is
`Bootstrapped`, with no change in flight and the peer still a member. Naming any other peer, or with an addition in
flight, it is answered `AbortIgnored`, and nothing changes.

**`RemovalCommitted`** makes its peer no member: `MemberRemoved`. The layer above reports a commit to every node,
whatever its stage, and every node applies it alike; a commit is never refused. The peer leaves every list the node
holds — its members, its pinging and collecting peers, its suspected peers and its missed periods — and if it was the
member probed last, the node has probed no one. On the coordinator, a commit naming the peer of the removal in flight
ends it, and the node is `Bootstrapped`; a commit naming another peer leaves the change in flight. A peer that is not a
member is answered `NonMemberIgnored`, and nothing changes but the epoch. A commit naming another peer never moves a
node out of `Initial`. A commit the node holds already is ignored, its own removal's included, and every other moves
its epoch (§16).

**A node's own removal leaves it `Removed`.** `RemovalCommitted` naming the node itself is answered `MemberRemoved`, and
the node is `Removed`: it is no longer among its members nor among the peers it gathered, it suspects no one, counts no
missed period, has probed no one and has no change in flight. The rest it keeps as it was: its other members, the peers
it gathered, its required count, its epoch, whether its pinging was completed and whether it missed its deadline. A
removed node admits a membership and `Probe` alone, and only a membership that names it takes it anywhere else (§16):
no member waits on it, as every member applies the same commit
([`P1-ADR-TerminalStatesAreNotSinks`](../docs/ADRs/P1-ADR-TerminalStatesAreNotSinks.md)).

**A removal changes who may be counted, never how many are required** (§13). On a concluded node, the removal of a
member not suspected may lose its quorum, and no removal restores it (§15).

## 13. The required count

**`RequiredCountChanged`** makes its `required_count` the count in effect: `RequiredCountSet`, carrying it. The caller
computes the count, as it computed the configuration's, and reports it when the change it belongs to commits; any count
is one (§2). A new count is a commit: one the node holds already is ignored, and every other moves its epoch (§16).

**In `Collecting`, a count the collecting peers already reach concludes**: `RequiredCountSet`, then `Concluded` with the
mode `Bootstrapped`, and the node is `Bootstrapped`. Elsewhere the count only takes effect: before the node's own
completion it is the count its completion checks (§8), and once the node has concluded it is the count its quorum is
held against, which it may lose or restore (§15).

## 14. Liveness

**A concluded node tracks its members' liveness**, one probe period at a time
([`P1-ADR-LivenessReportedNotStaged`](../docs/ADRs/P1-ADR-LivenessReportedNotStaged.md)). The caller keeps the clock:
it reports when a period starts, when a member is heard from, and when a member has missed its probe, directly or through
others. The node decides whom to probe, whom to ask, and when to suspect. Every member is alive when the node
concludes, and a member admitted or added later is alive when it becomes one. Liveness never changes the stage, and
nothing is ever removed for it: a suspected member stays a member until its removal commits (§12). A removed member's
liveness goes with it: it is no longer suspected, has missed no period, and if it was the member probed last the node
has probed no one.

**`ProbeDue`** names the member to probe: `SendProbe`, naming the member after the one probed last, in the members'
order, past the last member back to the first, the node itself skipped — the first member that is not the node, if it
has probed no one — and that member is the one probed last. A suspected member is probed as any other. A node that is
its only member is answered `NothingToProbe`, and nothing changes.

**`PeerHeard`** says the caller heard from the member: an answer to a probe, direct or relayed, or anything else the
caller counts as a sign of life. A suspected member is revived, `PeerRevived`, and is no longer suspected; any other
member is answered `AliveNoted`, and has missed no period.

**`ProbeMissed`** says the member missed its probe: `SendIndirectProbes`, the caller asking members of its choosing among
those listed to probe it. `via` lists every member not suspected but the node and the missed member, in the members'
order from the one after the missed member, past the last back to the first. Nothing changes.

**`IndirectProbesMissed`** says the member missed the period, directly and through every member asked. The member has
missed one period more: `MissCounted`, carrying how many periods running it has missed. Once it has missed as many as
the suspicion threshold, it is suspected instead: `PeerSuspected`, it becomes the last suspected peer, and it has missed
no period, being suspected.

**A member already suspected** is answered `AlreadySuspected` by `ProbeMissed` and `IndirectProbesMissed`, and nothing
changes. **A command of liveness naming the node itself** is answered `SelfIgnored`, and one naming a peer that is not a
member `NonMemberIgnored`; neither changes anything.

## 15. Quorum

**A concluded node holds its quorum while its members not suspected number the required count or more.** The node is
never suspected, and counts itself. A node concludes with every member alive and at least the required count of them,
so it concludes holding its quorum.

**A command that moves the node across its quorum says so**, last among its outcomes: `QuorumLost` when the node held its
quorum and holds it no longer, `QuorumRestored` when it had lost it and holds it again. A suspicion, a raised count,
the removal of a member not suspected or a membership adopted may lose it; a revival, a member admitted or added, a
lowered count or a membership adopted may restore it; no removal restores it. A command that does not cross
says nothing of it.

## 16. The epoch

**The epoch is the version of the membership a node holds** — its members and its required count
([`P1-ADR-TheEpochComesWithTheCommit`](../docs/ADRs/P1-ADR-TheEpochComesWithTheCommit.md)). The layer that commits
changes above the node numbers them, strictly increasing from one, and a commit carries its number: the node takes it,
and never counts. A node's epoch is its configuration's to begin with, and moves with the commits it applies and the
memberships it adopts, never back. A join decision does not move it: an admission is the caller's on each node, and the
epoch versions what the layer above commits. Where a form carries no epoch it is at epoch 0, which no commit is: a
caller that numbers nothing gives every node and every command epoch 0, and the node answers it as 0.4.1 did.

**A commit the node holds already is ignored.** `AdditionCommitted`, `RemovalCommitted` and `RequiredCountChanged` at
an epoch of one or more, at or below the node's, are answered `CommitIgnored`, carrying the commit's epoch, and nothing
changes: the node holds that membership already, or a later one. This comes before any other answer to a commit, the
node's own removal's included, so a commit reported twice, or after the node adopted a membership past it, is applied
once.

**The layer above reports its commits to a node in epoch order.** It may report one late, or twice; it never reports
one after a commit at a higher epoch. A commit reported out of order is held already by the node, ignored, and its
change lost to it — epochs need not be contiguous, so the node cannot see the gap — which is outside this
specification. A caller that cannot keep the order hands the node a membership instead (`MembershipReceived`), which
carries the whole of it.

**Every other commit moves the epoch.** A commit the node accepts and does not ignore leaves the node at the commit's
epoch, if it is above the node's, whatever else its row says — a commit answered `DuplicateMemberIgnored` or
`NonMemberIgnored` among them.

**A claim from behind is refused.** A node's own signals carry its epoch: participation, readiness and an addition's
acknowledgement. A member's participation or readiness, or the acknowledgement of the peer whose addition is requested,
at an epoch below the node's is refused: `StaleEpochIgnored`, carrying the peer and its epoch, then `SendMembership` —
the caller sends the peer the members, the required count and the epoch the view shows. It counts toward nothing and
changes nothing, but that an `Initial` node is now `Pinging`. A non-member's signal is ignored as before, whatever its
epoch (§6, §7), and so is an acknowledgement not awaited (§11). A claim at the node's epoch or above is answered as
before: the node behind is this one, and the membership will reach it. Liveness carries no epoch: being alive is not a
claim on a membership, and a member slow to apply a commit is never suspected for it.

**`MembershipReceived`** hands the node a membership: members, a required count and their epoch. It is admitted in
every stage. Whether the membership is genuine is the caller's to check before it reports it
([`P0-ADR-ProtocolAgnostic`](../docs/ADRs/P0-ADR-ProtocolAgnostic.md)). At or below the node's epoch it is answered
`MembershipIgnored`, carrying its epoch, and nothing changes.

**A membership above the node's epoch is adopted:** `MembershipAdopted`, carrying its epoch. The node's members,
required count and epoch are the membership's. Every list it holds keeps only the new members, in its order: its
pinging and its collecting peers, its suspected peers and its missed periods; if the member probed last is not one,
the node has probed no one; a member new to it is alive. Then:

- **Not naming the node**, the membership removes it, as its own removal's commit would (§12): `MemberRemoved`, naming
  the node, and the node is `Removed`. A node already `Removed` stays so, answered `MembershipAdopted` alone.
- **Naming a `Removed` node**, it is the node's way back: the node is `Initial`, a fresh node holding the membership —
  no pinging or collecting peer, its deadline not missed, its pinging not completed, no change in flight and no
  liveness tracked — which bootstraps among the members again.
- **Naming the node in any other stage**, the stage is kept, and so is what the node gathered among the new members:
  - `Collecting`, with the collecting peers numbering the new required count or more, concludes: then `Concluded`,
    mode `Bootstrapped`, and the node is `Bootstrapped`;
  - `Reconfiguring` gives up its change in flight, which the membership has moved past: then `ChangeAbandoned`,
    carrying the change as the view showed it, and the node is `Bootstrapped`, with no change in flight;
  - a concluded node's quorum is checked against the new members and count, and a crossing said last (§15).

## 17. Conformance

**A vector is a configuration, steps, and a final answer.** Each step is a command and the result it must be
answered with. The final answer is what a `Probe` issued after the last step must be answered: the view and the
admissible commands it is `Probed` with. A vector states nothing of an implementation's representation: it checks
state the way a caller would.

**Each vector names the rows of the matrix it exercises.** Every vector is valid against the schema's `vector`; the
matrix is valid against its `matrix`; [`released.json`](released.json) is valid against its `released`; and every
row of the matrix is exercised by at least one vector.

**Released vectors are kept.** `released.json` lists every vector of every released version by its file's name, the
version that released its bytes and the SHA-256 digest of them, and every command a released version names, by its
name and the first released version to name it. A listed vector is never removed or edited, and a listed command
never removed.

**An implementation conforms** when, created from each vector's configuration with an observer that does nothing
and given each step's command in turn, it answers each with the step's result, and the closing `Probe` with the
final answer — for a vector `released.json` lists, the admissible commands among those its version names (§1); and
when, created with an observer that keeps what it is told, it has reported each of those calls with the result it
answered, in order, and nothing else.
