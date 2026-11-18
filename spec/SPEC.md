# faction — the specification

**Version 0.9.0.** The four parts of this specification are versioned together: this prose, the data model in
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
epoch order (§16). 0.6.0 grows by the rule again: a newcomer holds its membership provisionally until its own addition
commits, and refuses a claim from behind without sending it (§16); every vector 0.5.x released is answered alike.
0.7.0 has a provisional node coordinate no change, refusing an addition or a removal it is asked for (§16), and
withdraws the one released vector that showed one coordinating (§1, §18); every other vector 0.6.0 released is
answered alike. 0.8.0 answers every call with a numbered receipt and names every node an instance (§2, §3), and
re-encodes every vector in force so, once, changing no answer (§1, §18). 0.9.0 states what the layer above owes a
proposal — to commit it only while no change committed since conflicts with it — and has the coordinator decide a
declined proposal again (§17); lets a coordinator hold several changes in flight, up to a bound its configuration
states, in a new stage, `Coordinating` (§2, §11, §12); and re-encodes every vector in force once more, the view's
changes in flight a list (§1, §4). Every vector 0.8.0 released is answered alike.

## 1. Conventions

**Every number is a string of decimal digits**, with no sign, no leading zero and no exponent, its value within
`0..=18446744073709551615`. Many JSON parsers read numbers as 64-bit floats, exact only to 2⁵³, and a peer id is
any u64.

**A node compares peer ids for equality, and does nothing else with one**: it never orders a peer id, reads its value
or computes with it, and keeps every list of peers in arrival order. An implementation may take
peer ids of any type it can copy and compare; this specification writes them as numbers.

**Every value of an enumeration is an object with exactly one member**, named for the value:
`{"ParticipationObserved": {"peer_id": "2"}}`, `{"Concluded": {"mode": {"Bootstrapped": {}}}}`. A value that carries
nothing carries the empty object: `{"Probe": {}}`, `{"Fresh": {}}`.

**Names are the vocabulary's.** A command says what the caller observed — a peer's signal, its own completion, the
deadline, a join decision, an addition's or a removal's progress, a proposal declined, a new required count, a
membership handed to it, a probe period, a member heard from or missed — or, `Probe`, asks. An outcome says what happened to the signal, or, in the imperative, what the caller must now do:
`BroadcastLocalReady`, `AcknowledgeRejoin`, `EmitJoinRequest`, `SendMembership`, `ProposeAddition`, `ProposeRemoval`,
`SendProbe`, `SendIndirectProbes`.

**A list of commands by name** is in the commands' order: `ParticipationObserved`, `ReadyObserved`,
`LocalParticipationCompleted`, `DeadlineExpired`, `JoinRequested`, `JoinApproved`, `JoinRejected`, `AdditionRequested`,
`MembershipAcknowledged`, `AdditionCommitted`, `AdditionAborted`, `RemovalRequested`, `RemovalCommitted`,
`RemovalAborted`, `ProposalDeclined`, `RequiredCountChanged`, `MembershipReceived`, `ProbeDue`, `PeerHeard`,
`ProbeMissed`, `IndirectProbesMissed`, `Probe`.

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
that version or an earlier one (§18). Every answer this version gives is therefore given the same by every later
version, and every vector released here passes unchanged there — but one a later version withdraws. A version
withdraws a released vector only to correct a defect the vector shows, never to grow: `released.json` marks it
withdrawn at that version, and its bytes move, unchanged, from [`vectors/`](vectors/) to [`withdrawn/`](withdrawn/)
(§18). 0.7.0 withdraws vector 113, in which a provisional node coordinated an addition, offering the newcomer a
membership no commit made (§16). A version re-encodes the released vectors only all at once, and only to change
how they are written, never an answer: `released.json` marks each with every version that re-encoded it, in order,
and the digest of the bytes each wrote, and the bytes each re-encoding replaced are kept, unchanged, under
[`released/`](released/), by the last version that wrote them (§18). 0.8.0 re-encodes the 115 vectors in force: each step's result is its receipt's
output, numbered by the step's position, the closing answer the receipt the closing `Probe` is given, and the
configuration's peer id its instance's node, at incarnation 1; their bytes as 0.7.0 wrote them are under
[`released/0.7.0/`](released/0.7.0/). 0.9.0 re-encodes them once more: the view's `reconfiguration` becomes
`reconfigurations`, the changes in flight as a list — `None` the empty list, any other change a list of it alone;
their bytes as 0.8.0 wrote them are under [`released/0.8.0/`](released/0.8.0/). 0.5.0 grows by the rule: the epoch (§16), a member of
the configuration, the view, three signals and three commits, is omitted wherever it is 0, and a form with no epoch
holds epoch 0; the command `MembershipReceived` is new, and so are the outcomes it and the epoch bring. 0.6.0 grows by
it again: whether a node's membership is provisional (§16), a member of the configuration and of the view, is omitted
wherever it is not. 0.9.0 grows by it once more: the command `ProposalDeclined` is new, and so is the outcome
`DeclineIgnored` it brings (§17); the configuration's bound of changes in flight, `changes_in_flight`, is omitted
wherever it is 1; and the stage `Coordinating`, which a node bound to one change never reaches, is new, and so is the
outcome `DuplicateChangeIgnored` (§2, §11, §12). Before the rule was written the encoding grew in place: 0.2.0,
0.3.0 and 0.4.0 each added fields to the view and the configuration, and commands to what earlier stages admit,
rewriting the vectors released before them -- never an outcome -- and 0.4.0 renamed the view's `addition` to
`reconfiguration`, which 0.9.0 made a list.

## 2. A node

**A node is created from its configuration**, and with an observer (§3). The configuration is the node's **instance**
— its node, its own peer id, and an incarnation the caller gives it (§3) — its genesis peers, a required count, a
suspicion threshold and an epoch, 0 unless it states another: the epoch of the membership its peers and count are
(§16); and whether that membership is provisional, which it is not unless the configuration says so: a newcomer's,
created from the membership a coordinator offered it, before its own addition commits (§16). It states, too, the
node's **bound**: how many changes it may hold in flight at once as a coordinator, 1 unless it states another (§11,
§12); it is at least one, and a configuration with none is outside this specification. The genesis peers include
the node itself and list no peer twice; a configuration that breaks either is outside this specification. The required
count is how many confirmations conclude bootstrapping, and any count is one: it is never derived from the number of
peers ([`P0-ADR-ProtocolAgnostic`](../docs/ADRs/P0-ADR-ProtocolAgnostic.md)). The suspicion threshold is how many
periods running a member must miss before the node suspects it (§14); it is at least one, and a configuration with
none is outside this specification. The configuration never changes
([`P1-ADR-ConfigIsImmutableGenesis`](../docs/ADRs/P1-ADR-ConfigIsImmutableGenesis.md)); the count in effect starts as
the configuration's, and changes by its own command alone (§13).

**A node is in one of seven stages:** `Initial`, created and given nothing yet; `Pinging`, gathering its peers'
participation; `Collecting`, its own participation completed, gathering readiness; `Bootstrapped`, concluded;
`Coordinating`, concluded with changes in flight, fewer than its bound; `Reconfiguring`, concluded with as many
changes in flight as its bound — each an addition (§11) or a removal (§12); and `Removed`, removed from its own
cluster (§12).
A node is created `Initial`, leaves it on the first accepted command that is not a membership decision — a join
decision, a committed addition or removal, a new required count, or a membership handed to it — or on its own removal,
and returns to it from `Removed` alone, adopting a membership that names it (§16).
`Bootstrapped` is the only conclusion: a node that has concluded is `Bootstrapped`, or `Coordinating` or
`Reconfiguring` while changes are in flight, until it is removed, and no command takes it back to an earlier stage. A
removed node stays `Removed` until it adopts a membership that names it.

**The stage the changes in flight make** a concluded node's is `Bootstrapped` with none, `Coordinating` with fewer
than its bound, and `Reconfiguring` with as many: a node bound to one change is never `Coordinating`. A change that
starts or ends leaves the node in the stage its changes in flight then make.

**Besides its stage a node holds** its members, its pinging peers, its collecting peers, whether it has missed its
deadline, the required count in effect, its changes in flight, in the order requested, its suspected peers, the periods each member has missed,
the member it probed last, its epoch, and whether its membership is provisional. A fresh node's members are its genesis
peers; it has no pinging or collecting peer, has not missed its deadline, its required count is the configuration's, it
has no change in flight, suspects no one, counts no missed period, has probed no one, its epoch is the configuration's,
and its membership is provisional as the configuration says. Members are added (§10, §11),
removed (§12) and replaced with a membership the node adopts (§16).

## 3. One entry point, one receipt

**Every call is a command, and every command is answered with a receipt**: an output and a number. The output is
the call's result: `Accepted`, carrying the outcomes, the view and the admissible commands; `Rejected`, carrying the
view and the admissible commands; or `Probed`, carrying the same two
([`P1-ADR-SingleEntryPoint`](../docs/ADRs/P1-ADR-SingleEntryPoint.md)). The view and the admissible commands a result
carries are the node's after the call. The number is one more than the instance's previous receipt's, the first being
1, whatever the output: a rejected command and a probe are numbered like the rest. It is a count the instance keeps
itself, needing no clock: the same configuration and the same commands give the same numbers.

**An instance is a node and an incarnation.** The caller guarantees an incarnation never repeats for its node: a node
created again — restarted from its genesis or from a membership, or a removed node made a newcomer again — is a new
instance, under a new incarnation, and numbers from 1. The instance tells one instance's receipts and log from
another's; that an incarnation is new is the caller's to keep, never checked.

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

**Every call is reported to the node's observer**, given when the node is created: the command and its receipt,
exactly once, by the time the receipt is returned — a rejected command and a probe like the rest. The observer is
told and answers nothing, so it cannot change a result; a node with no use for one is given an observer that does
nothing ([`P2-ADR-TotalObservability`](../docs/ADRs/P2-ADR-TotalObservability.md)).

**A node's log is its configuration, then one step per call** — the command and its receipt — in the order the calls
were made, as its observer was told them. A gap in the receipts' numbers proves a line lost. Created from the
configuration and given each step's command in turn, a conforming node answers each step's receipt: a log that does
not replay was cut, reordered or altered, or written by a node that does not conform.

## 4. The view

**The view is what a node shows of its state**: its peer state, whether its pinging is completed, its pinging
peers, its collecting peers, its required count, whether it has missed its deadline, its members, its reconfigurations, its
suspected peers, its missed periods, the member it probed last, its quorum, its epoch (§16), omitted at 0, and whether
its membership is provisional (§16), omitted when it is not.

**The peer state** is `Fresh` while the node is `Initial`, `Bootstrapped` once it has concluded — `Bootstrapped`,
`Coordinating` or `Reconfiguring` — `Removed` once it is removed, and otherwise its stage, `Pinging` or `Collecting`, or `TimedOut` if it has missed its deadline. A missed deadline is a fact the view
reports, not a stage: a `TimedOut` node goes on as its stage does, and may still conclude, after which its peer
state is `Bootstrapped` and its view still says the deadline was missed
([`P1-ADR-TerminalStatesAreNotSinks`](../docs/ADRs/P1-ADR-TerminalStatesAreNotSinks.md)).

**Pinging is completed** once the node's own participation is: in `Collecting`, `Bootstrapped`, `Coordinating` and
`Reconfiguring`,
not before; in `Removed`, as it was when the node was removed.

**The required count** is the count in effect (§13). A node is concluded when its peer state is `Bootstrapped`.

**The reconfigurations** are the changes in flight, in the order they were requested, and none with no change in
flight. An addition is `AdditionRequested`, naming the peer, once it is requested, and `AdditionAcknowledged`, naming
it, once the peer has acknowledged the membership (§11); a removal is `RemovalRequested`, naming the peer, once it is
requested (§12).

**The suspected peers** are the members the node suspects (§14). **The missed periods** list each member that has
missed one or more periods running and is not suspected, with how many. **The member probed last** is `None` until the
node first probes, then `Probed`, naming the member.

**The quorum** is `NotTracked` until the node has concluded, and once it is removed; while it is concluded, `Held` while its members not suspected number
the required count or more, and `Lost` while they number fewer (§15).

## 5. Admission

Each stage admits these commands, and rejects the rest:

| Stage | Admits |
|---|---|
| `Initial` | every command but `DeadlineExpired`, `AdditionRequested`, `MembershipAcknowledged`, `AdditionAborted`, `RemovalRequested`, `RemovalAborted`, `ProposalDeclined` and the four of liveness |
| `Pinging` | every command but `AdditionRequested`, `MembershipAcknowledged`, `AdditionAborted`, `RemovalRequested`, `RemovalAborted`, `ProposalDeclined` and the four of liveness |
| `Collecting` | every command but `LocalParticipationCompleted`, `AdditionRequested`, `MembershipAcknowledged`, `AdditionAborted`, `RemovalRequested`, `RemovalAborted`, `ProposalDeclined` and the four of liveness |
| `Bootstrapped` | `ParticipationObserved`, `JoinRequested`, `JoinApproved`, `JoinRejected`, `AdditionRequested`, `AdditionCommitted`, `RemovalRequested`, `RemovalCommitted`, `RequiredCountChanged`, `MembershipReceived`, `ProbeDue`, `PeerHeard`, `ProbeMissed`, `IndirectProbesMissed` and `Probe` |
| `Coordinating` | what `Reconfiguring` admits, and `AdditionRequested` and `RemovalRequested` |
| `Reconfiguring` | `ParticipationObserved`, `JoinRequested`, `JoinRejected`, `MembershipAcknowledged`, `AdditionCommitted`, `AdditionAborted`, `RemovalCommitted`, `RemovalAborted`, `ProposalDeclined`, `RequiredCountChanged`, `MembershipReceived`, `ProbeDue`, `PeerHeard`, `ProbeMissed`, `IndirectProbesMissed` and `Probe` |
| `Removed` | `MembershipReceived` and `Probe` |

The four of liveness are `ProbeDue`, `PeerHeard`, `ProbeMissed` and `IndirectProbesMissed`. A node completes its own
participation once; a concluded node answers only a peer still trying to bootstrap, a newcomer, an addition, a removal, a
new count, a membership and liveness; an addition or a removal is requested of a concluded node alone, below its bound, and
a proposal's decline reaches a node with a change in flight alone; a commit and a new count reach every node, whatever its stage, until it is removed, and a membership reaches every
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

**After it** — `Collecting`, `Bootstrapped`, `Coordinating` or `Reconfiguring` — a member's participation means that peer is still
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
`Removed`, and `JoinApproved` in every stage but `Coordinating`, `Reconfiguring` and `Removed`: no admission goes
around a change in flight.

**`JoinRequested`** is answered `EmitJoinRequest`: the caller forwards the newcomer's request to whoever decides.
Nothing changes; a repeated request is forwarded again.

**`JoinApproved`** admits the peer: `MemberAdmitted`, and the peer becomes the last member. A peer that is already a
member — the node itself among them — is answered `DuplicateMemberIgnored`, and nothing changes.

**`JoinRejected`** is answered `JoinDenied`, and nothing changes: a peer that is a member stays one.

**Admission changes who may be counted, never how many are required** (§13). On a concluded node, a member admitted
may restore its quorum (§15).

## 11. Addition

**A concluded node grows by one through the layer that commits changes above it**, up to its bound of changes in
flight at a time (§2).
The node that coordinates an addition is asked for it, hands the newcomer the membership, and tells the caller when to
propose it; every node applies it once it has committed. Who may be added, and how a change commits, are the caller's;
what the layer above owes a proposal is §17.

**`AdditionRequested`** starts an addition: `SendMembership` — the caller sends the newcomer the members, the
required count and the epoch the view shows — and the addition, `AdditionRequested`, is the last of its changes in
flight: the node is in the stage they make (§2). A peer that is already a member, the node itself among them, is
answered `DuplicateMemberIgnored`, and nothing changes. A node whose membership is provisional coordinates no addition:
it is answered `CoordinationRefused`, carrying the peer, before anything else, and nothing changes (§16).

**No two changes in flight name one peer.** A request, of either kind, naming a peer with a change in flight is
answered `DuplicateChangeIgnored`, after the provisional refusal and before any other answer, and nothing changes.
`Reconfiguring` admits no request: a node at its bound takes no change more.

**`MembershipAcknowledged`** from the peer whose addition is requested is answered `ProposeAddition` — the caller
proposes the addition to the layer above, decided at the epoch the view shows (§17) — and the addition is
`AdditionAcknowledged`. From a peer whose addition is not requested — none in flight, or acknowledged already — it is
answered `AcknowledgementIgnored`, and nothing changes. From the peer
whose addition is requested, behind — its `epoch` below the node's — it is refused (§16).

**`AdditionAborted`** naming a peer whose addition is in flight withdraws it: `AdditionWithdrawn`, the change is in
flight no more, the node is in the stage the others make (§2), and the peer is no member. Naming a peer whose addition
is not in flight, it is answered `AbortIgnored`, and nothing changes.

**`AdditionCommitted`** makes its peer the last member: `MemberAdmitted`. The layer above reports a commit to every
node, whatever its stage, and every node applies it alike. On the coordinator, a commit naming the peer of an addition
in flight ends that change, and the node is in the stage the others make (§2); every other change in flight stays. A
peer that is already a member is answered `DuplicateMemberIgnored`, and nothing changes but the epoch. A
commit the node holds already is ignored, and every other moves its epoch (§16). The peer of an
addition in flight is never a member: its request is refused if it is one, and no admission is admitted meanwhile.

**An addition changes who may be counted, never how many are required** (§13). On a concluded node, a member added
may restore its quorum (§15).

## 12. Removal

**A concluded cluster shrinks by one through the layer that commits changes above it**, up to the coordinator's bound
of changes in flight at a time, additions and removals (§2)
([`P1-ADR-RemovalThroughTheLayerThatCommits`](../docs/ADRs/P1-ADR-RemovalThroughTheLayerThatCommits.md)). The node that coordinates a removal is asked for it and tells the caller to propose it at
once — a member being removed has nothing to acknowledge, and may not answer at all; every node applies it once it has
committed. Who may be removed, and how a change commits, are the caller's: a suspected member stays a member until its
removal commits (§14).

**`RemovalRequested`** starts a removal: `ProposeRemoval` — the caller proposes the removal to the layer above, decided
at the epoch the view shows (§17) — and the removal, `RemovalRequested`, is the last of its changes in flight: the node
is in the stage they make (§2). Any member may be named, the node itself among them. A peer that is not a member is
answered `NonMemberIgnored`, and nothing changes. A node whose membership is provisional coordinates no removal: it is
answered `CoordinationRefused`, carrying the peer, before anything else, and nothing changes (§16). A peer with a
change in flight is answered `DuplicateChangeIgnored` (§11), and `Reconfiguring` admits no request.

**A removal that would lose the quorum is refused.** The removal of a member not suspected, when without it and every
member whose removal is in flight the members not suspected would number fewer than the required count, is answered
`RemovalRefused`, and nothing changes: the removals in flight are counted together. The node counts itself and is
never suspected (§15). A suspect's removal is never refused for the quorum: it leaves the members
not suspected as they are. The node decides on its own view.

**`RemovalAborted`** naming a peer whose removal is in flight withdraws it: `RemovalWithdrawn`, the change is in
flight no more, the node is in the stage the others make (§2), and the peer is still a member. Naming a peer whose
removal is not in flight, it is answered `AbortIgnored`, and nothing changes.

**`RemovalCommitted`** makes its peer no member: `MemberRemoved`. The layer above reports a commit to every node,
whatever its stage, and every node applies it alike; a commit is never refused. The peer leaves every list the node
holds — its members, its pinging and collecting peers, its suspected peers and its missed periods — and if it was the
member probed last, the node has probed no one. On the coordinator, a commit naming the peer of a removal in flight
ends that change, and the node is in the stage the others make (§2); every other change in flight stays. A peer that is
not a
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

**A provisional node sends no membership, and coordinates no change**
([`P1-ADR-ANewcomerHoldsItsMembershipProvisionally`](../docs/ADRs/P1-ADR-ANewcomerHoldsItsMembershipProvisionally.md)).
A newcomer holds the membership its coordinator offered it, and itself,
before the layer above has committed its addition: its membership is provisional, and a peer adopting it would hold a
member no commit added. A node whose membership is provisional refuses a claim from behind as any node does,
`StaleEpochIgnored`, carrying the peer and its epoch, and sends the peer nothing: a member that holds a committed
membership will. Asked for an addition or a removal, it refuses, `CoordinationRefused`, carrying the peer, and nothing
changes (§11, §12): an addition it coordinated would hand the newcomer a membership naming a member no commit added,
and a change it coordinates is the cluster's, of which it is no member of record yet. So it never has a change in
flight. Its membership stops being provisional, once and for all, when the node applies the commit of its own
addition — at an epoch above its own, whatever else its row says, `DuplicateMemberIgnored` among them — when it adopts
a membership, and when it is removed (§12). Nothing else changes it.

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
  - `Coordinating` and `Reconfiguring` give up every change in flight, which the membership has moved past: then
    `ChangeAbandoned` for each, in the order requested, carrying the change as the view showed it, and the node is
    `Bootstrapped`, with no change in flight;
  - a concluded node's quorum is checked against the new members and count, and a crossing said last (§15).

## 17. Proposals

**A proposal is decided at the epoch its answer's view shows**
([`P1-ADR-AProposalIsCommittedOnlyWhileValid`](../docs/ADRs/P1-ADR-AProposalIsCommittedOnlyWhileValid.md)).
`ProposeAddition` and `ProposeRemoval` tell the caller to propose a change to the layer above, and the view the answer
carries shows the membership the node decided it on, at its epoch: the addition of a peer that is not a member (§11),
the removal of a member its quorum can spare (§12).

**What the layer above owes a proposal: to commit it only while it is still valid.** A proposal decided at epoch `e`
is valid at the layer above's epoch `c` when no change it committed at an epoch above `e`, up to `c`, conflicts with
it:

- a change naming the same peer conflicts with it;
- for a removal, another removal or a new required count conflicts with it too: either moves what the removal's quorum
  check counted;
- nothing else conflicts: an addition of another peer only adds to the members a removal spares, and an addition
  checks no quorum.

A valid proposal holds at `c` what the node decided it on at `e`. The rule reads the committed changes alone, never a
node's suspicions, so any layer above can keep it. A layer above may decline more than the rule asks — every proposal
decided before its latest commit, say — but commits no proposal that is not valid. A proposal it commits it reports as
it reports any commit (§16); one it declines it reports to the coordinator alone, `ProposalDeclined`, carrying the
epoch it declined it at, once it has reported every commit up to that epoch to the coordinator: as it reports its
commits in epoch order (§16), the node then decides again on a view that holds them.

**`ProposalDeclined`** reports a proposal declined at the layer above's `epoch`, and the node decides the change again
on its present view. It is admitted in `Coordinating` and `Reconfiguring` alone (§5): a decline is the
coordinator's, of a change it has in flight.

- **Naming no proposal in flight** — no change of `peer_id` is in flight, or its addition is not yet acknowledged, and
  so not yet proposed — it is answered `DeclineIgnored`, and nothing changes.
- **An addition** is proposed again: `ProposeAddition`, at the node's epoch, and the change stays in flight. Its peer
  is never a member while its addition is in flight (§11), so on the node's own view it is always still valid.
- **A removal** the quorum can still spare is proposed again: `ProposeRemoval`, at the node's epoch, and the change
  stays in flight. One it can no longer spare — of a member not suspected, without which and every other member whose
  removal is in flight the members not suspected would number fewer than the required count — is given up:
  `RemovalRefused`, the change is in flight no more, the node is in the stage the others make (§2), and the peer is
  still a member. A suspect's removal is never given up for the quorum (§12).

The decline's epoch is the layer above's, and the node decides on the view it holds, whatever the epoch: a decline
reported before the commits it was declined for has the node decide without them, and a change it proposes again the
layer above declines again, until they reach it.

## 18. Conformance

**A vector is a configuration, steps, and a final answer.** Each step is a command and the receipt it must be
answered with. The final answer is the receipt a `Probe` issued after the last step must be given: numbered one more
than the last step's, the view and the admissible commands it is `Probed` with. A vector states nothing of an implementation's representation: it checks
state the way a caller would.

**Each vector names the rows of the matrix it exercises.** Every vector is valid against the schema's `vector`; the
matrix is valid against its `matrix`; [`released.json`](released.json) is valid against its `released`; and every
row of the matrix is exercised by at least one vector.

**Released vectors are kept.** `released.json` lists every vector of every released version by its file's name, the
version that released its bytes and the SHA-256 digest of them, and every command a released version names, by its
name and the first released version to name it. A listed vector is never edited but by a re-encoding (§1), and a
listed command never removed. A re-encoded vector is marked with the version that re-encoded it and its re-encoded
bytes' digest — every re-encoding, in order — and the bytes each replaced are kept under [`released/`](released/):
it answers every step as they do. A listed vector a later version withdraws to correct a defect (§1) is marked withdrawn at that version, and its
bytes move, unchanged, to [`withdrawn/`](withdrawn/): no implementation is held to it. [`vectors/`](vectors/) holds
every vector in force.

**An implementation conforms** when, created from each vector's configuration with an observer that does nothing
and given each step's command in turn, it answers each with the step's receipt, and the closing `Probe` with the
final answer — for a vector `released.json` lists, the admissible commands among those its version names (§1); and
when, created with an observer that keeps what it is told, it has reported each of those calls with the receipt it
answered, in order, and nothing else.
