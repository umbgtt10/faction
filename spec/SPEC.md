# faction — the specification

**Version 0.2.0.** The four parts of this specification are versioned together: this prose, the data model in
[`schema.json`](schema.json), the decision table in [`matrix.json`](matrix.json) and the conformance vectors in
[`vectors/`](vectors/). It is normative; the `faction` crate is one implementation of it, and where the two disagree
the specification is right ([`P0-ADR-SpecificationIsNormativeCrateIsReference`](../docs/ADRs/P0-ADR-SpecificationIsNormativeCrateIsReference.md)).

**What 0.2.0 covers:** faction whole as it stands after its first phase and its addition — a node created from its
configuration, and the observer it reports every call to; bootstrapping, from the first signal to the conclusion, with
the deadline; joining, a newcomer admitted or denied in every stage; addition, a concluded node growing by one through
the layer that commits changes above it; the required count in effect; and the probe. Each command with every row it
has.

## 1. Conventions

**Every number is a string of decimal digits**, with no sign, no leading zero and no exponent, its value within
`0..=18446744073709551615`. Many JSON parsers read numbers as 64-bit floats, exact only to 2⁵³, and a peer id is
any u64.

**Every value of an enumeration is an object with exactly one member**, named for the value:
`{"ParticipationObserved": {"peer_id": "2"}}`, `{"Concluded": {"mode": {"Bootstrapped": {}}}}`. A value that carries
nothing carries the empty object: `{"Probe": {}}`, `{"Fresh": {}}`.

**Names are the vocabulary's.** A command says what the caller observed — a peer's signal, its own completion, the
deadline, a join decision, an addition's progress, a new required count — or, `Probe`, asks. An outcome says what
happened to the signal, or, in the imperative, what the caller must now do: `BroadcastLocalReady`, `AcknowledgeRejoin`,
`EmitJoinRequest`, `SendMembership`, `ProposeAddition`.

**A list of commands by name** is in the commands' order: `ParticipationObserved`, `ReadyObserved`,
`LocalParticipationCompleted`, `DeadlineExpired`, `JoinRequested`, `JoinApproved`, `JoinRejected`, `AdditionRequested`,
`MembershipAcknowledged`, `AdditionCommitted`, `AdditionAborted`, `RequiredCountChanged`, `Probe`.

**A list of peers is in arrival order.** The members are the configuration's peers in its order, then each admitted
or added peer as it is admitted or added; the pinging and the collecting peers are in the order their signals were accepted. The same
configuration and the same commands give the same lists, so two implementations' answers to the same calls are
identical, and a list never holds a peer twice
([`P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder`](../docs/ADRs/P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder.md)).

**Outcomes are a sequence**: their order is part of the answer.

## 2. A node

**A node is created from its configuration**, and with an observer (§3). The configuration is the node's own peer
id, its genesis peers and a required count. The genesis peers include the node itself and list no peer twice; a
configuration that breaks either is outside this specification. The required count is how many confirmations
conclude bootstrapping, and any count is one: it is never derived from the number of peers
([`P0-ADR-ProtocolAgnostic`](../docs/ADRs/P0-ADR-ProtocolAgnostic.md)). The configuration never changes
([`P1-ADR-ConfigIsImmutableGenesis`](../docs/ADRs/P1-ADR-ConfigIsImmutableGenesis.md)); the count in effect starts
as the configuration's, and changes by its own command alone (§12).

**A node is in one of five stages:** `Initial`, created and given nothing yet; `Pinging`, gathering its peers'
participation; `Collecting`, its own participation completed, gathering readiness; `Bootstrapped`, concluded; and
`Reconfiguring`, concluded with an addition in flight (§11).
A node is created `Initial`, leaves it on the first accepted command that is not a membership decision — a join
decision, a committed addition or a new required count — and never returns to it.
`Bootstrapped` is the only conclusion: a node that has concluded is `Bootstrapped`, or `Reconfiguring` while an
addition is in flight, and no command takes it back to an earlier stage.

**Besides its stage a node holds** its members, its pinging peers, its collecting peers, whether it has missed its
deadline, the required count in effect and the addition in flight. A fresh node's members are its genesis peers; it
has no pinging or collecting peer, has not missed its deadline, its required count is the configuration's, and it has
no addition in flight. Members are only ever added (§10, §11).

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

**An accepted command is answered `Accepted`**, with the outcomes and the change its row gives (§6–§12). An outcome
that names a peer names the command's `peer_id`; an outcome that carries a required count carries the command's.

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
peers, its collecting peers, its required count, whether it has missed its deadline, its members, and its addition.

**The peer state** is `Fresh` while the node is `Initial`, `Bootstrapped` once it has concluded — `Bootstrapped` or
`Reconfiguring` — and otherwise its stage, `Pinging` or `Collecting`, or `TimedOut` if it has missed its deadline. A missed deadline is a fact the view
reports, not a stage: a `TimedOut` node goes on as its stage does, and may still conclude, after which its peer
state is `Bootstrapped` and its view still says the deadline was missed
([`P1-ADR-TerminalStatesAreNotSinks`](../docs/ADRs/P1-ADR-TerminalStatesAreNotSinks.md)).

**Pinging is completed** once the node's own participation is: in `Collecting`, `Bootstrapped` and `Reconfiguring`,
not before.

**The required count** is the count in effect (§12). A node is concluded when its peer state is `Bootstrapped`.

**The addition** is `None` with no addition in flight; `Requested`, naming the peer, once its addition is requested;
and `Acknowledged`, naming it, once the peer has acknowledged the membership (§11).

## 5. Admission

Each stage admits these commands, and rejects the rest:

| Stage | Admits |
|---|---|
| `Initial` | every command but `DeadlineExpired`, `AdditionRequested`, `MembershipAcknowledged` and `AdditionAborted` |
| `Pinging` | every command but `AdditionRequested`, `MembershipAcknowledged` and `AdditionAborted` |
| `Collecting` | every command but `LocalParticipationCompleted`, `AdditionRequested`, `MembershipAcknowledged` and `AdditionAborted` |
| `Bootstrapped` | `ParticipationObserved`, `JoinRequested`, `JoinApproved`, `JoinRejected`, `AdditionRequested`, `AdditionCommitted`, `RequiredCountChanged` and `Probe` |
| `Reconfiguring` | `ParticipationObserved`, `JoinRequested`, `JoinRejected`, `MembershipAcknowledged`, `AdditionCommitted`, `AdditionAborted`, `RequiredCountChanged` and `Probe` |

A node completes its own participation once; a concluded node answers only a peer still trying to bootstrap, a
newcomer, an addition and a new count; an addition is requested of a concluded node alone, one at a time; and a commit
and a new count reach every node, whatever its stage.

**Admission is decided when a command arrives, never by what a caller can construct.** Every command can be offered
to a node in every stage, and every vector runs on every conforming implementation: none relies on a type system to
keep a command from a stage
([`P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors`](../docs/ADRs/P1-ADR-StaticInvariantsBecomeDynamicRejectionVectors.md)).

## 6. `ParticipationObserved`

**From a peer that is not a member**, the signal is ignored: `NonMemberIgnored`, and nothing changes, but that an
`Initial` node is now `Pinging`. A non-member's signal is not remembered: admitting the peer later does not count it.

**Before the node's own completion** — `Initial` or `Pinging` — a member's first participation is accepted,
`ParticipationAccepted`, and the peer becomes the last pinging peer; a repeat is ignored,
`DuplicateParticipationIgnored`. Either way the node is `Pinging`.

**After it** — `Collecting`, `Bootstrapped` or `Reconfiguring` — a member's participation means that peer is still
trying to bootstrap, and is answered `AcknowledgeRejoin`: the caller re-advertises the node's readiness to it. Nothing
changes. None of them is a sink ([`P1-ADR-CollectingIsNotASink`](../docs/ADRs/P1-ADR-CollectingIsNotASink.md),
[`P1-ADR-TerminalStatesAreNotSinks`](../docs/ADRs/P1-ADR-TerminalStatesAreNotSinks.md)).

## 7. `ReadyObserved`

**From a peer that is not a member**, the signal is ignored, as participation is (§6). A concluded node does not admit
readiness.

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

**Join decisions never change the stage.** `JoinRequested` and `JoinRejected` are admitted in every stage, and
`JoinApproved` in every stage but `Reconfiguring`: no admission goes around an addition in flight.

**`JoinRequested`** is answered `EmitJoinRequest`: the caller forwards the newcomer's request to whoever decides.
Nothing changes; a repeated request is forwarded again.

**`JoinApproved`** admits the peer: `MemberAdmitted`, and the peer becomes the last member. A peer that is already a
member — the node itself among them — is answered `DuplicateMemberIgnored`, and nothing changes.

**`JoinRejected`** is answered `JoinDenied`, and nothing changes: a peer that is a member stays one.

**Admission changes who may be counted, never how many are required** (§12).

## 11. Addition

**A concluded node grows by one through the layer that commits changes above it**, one addition in flight at a time.
The node that coordinates an addition is asked for it, hands the newcomer the membership, and tells the caller when to
propose it; every node applies it once it has committed. Who may be added, and how a change commits, are the caller's.

**`AdditionRequested`** starts an addition: `SendMembership` — the caller sends the newcomer the members and the
required count the view shows — and the node is `Reconfiguring`, the addition `Requested`. A peer that is already a
member, the node itself among them, is answered `DuplicateMemberIgnored`, and nothing changes. `Reconfiguring` does not
admit a second request.

**`MembershipAcknowledged`** from the peer whose addition is `Requested` is answered `ProposeAddition` — the caller
proposes the addition to the layer above — and the addition is `Acknowledged`. From any other peer, or once
acknowledged, it is answered `AcknowledgementIgnored`, and nothing changes.

**`AdditionAborted`** naming the peer of the addition in flight withdraws it: `AdditionWithdrawn`, and the node is
`Bootstrapped`, with no addition in flight and the peer no member. Naming any other peer, it is answered
`AbortIgnored`, and nothing changes.

**`AdditionCommitted`** makes its peer the last member: `MemberAdmitted`. The layer above reports a commit to every
node, whatever its stage, and every node applies it alike. On the coordinator, a commit naming the peer of the
addition in flight ends it, and the node is `Bootstrapped`; a commit naming another peer leaves the addition in
flight. A peer that is already a member is answered `DuplicateMemberIgnored`, and nothing changes. The peer of the
addition in flight is never a member: its request is refused if it is one, and no admission is admitted meanwhile.

**An addition changes who may be counted, never how many are required** (§12).

## 12. The required count

**`RequiredCountChanged`** makes its `required_count` the count in effect: `RequiredCountSet`, carrying it. The caller
computes the count, as it computed the configuration's, and reports it when the change it belongs to commits; any count
is one (§2).

**In `Collecting`, a count the collecting peers already reach concludes**: `RequiredCountSet`, then `Concluded` with the
mode `Bootstrapped`, and the node is `Bootstrapped`. Elsewhere the count only takes effect: before the node's own
completion it is the count its completion checks (§8), and once the node has concluded it changes the view alone.

## 13. Conformance

**A vector is a configuration, steps, and a final answer.** Each step is a command and the result it must be
answered with. The final answer is what a `Probe` issued after the last step must be answered: the view and the
admissible commands it is `Probed` with. A vector states nothing of an implementation's representation: it checks
state the way a caller would.

**Each vector names the rows of the matrix it exercises.** Every vector is valid against the schema's `vector`; the
matrix is valid against its `matrix`; and every row of the matrix is exercised by at least one vector.

**An implementation conforms** when, created from each vector's configuration with an observer that does nothing
and given each step's command in turn, it answers each with the step's result, and the closing `Probe` with the
final answer; and when, created with an observer that keeps what it is told, it has reported each of those calls with
the result it answered, in order, and nothing else.
