# Changelog

The crate's versions. The specification in [`spec/`](spec/SPEC.md) is versioned on its own since 0.5.0; each
release since names the version it conforms to.

## [Unreleased]

### Phase 6, first half: a proposal committed only while it is still valid

Conforms to the specification's version 0.9.0: every vector 0.8.0 released is answered alike, and five new ones,
117 to 121, show a proposal declined and decided again.

#### Added
- The specification states what the layer above owes a proposal (§17): it is decided at the epoch its answer's view
  shows, and the layer above commits it only while no change committed since conflicts with it -- one naming the same
  peer, or, for a removal, another removal or a new required count. The rule reads the committed changes alone.
- `Command::ProposalDeclined { peer_id, epoch }`: the layer above reports a proposal it declined, after the commits up
  to the epoch it declined it at, and the coordinator decides the change again on its present view -- an addition
  proposed again, a removal proposed again while the quorum can spare its member and given up, `RemovalRefused`, once
  it cannot. `Reconfiguring` admits it; every other stage rejects it.
- `Outcome::DeclineIgnored`: a decline naming no proposal in flight.
- `Bootstrapped::can_spare`, public: whether the quorum can spare a member, as a removal requested and a removal
  declined are both decided by it.
- faction-protocol: `Protocol::decline(peer_id, epoch)`, a declined proposal reaching the machine.

#### Changed
- **Breaking:** `Command` and `Outcome` gain a variant each.
- **Breaking:** faction-protocol's `OutputMessage::ProposeAddition` and `ProposeRemoval` carry the epoch the proposal
  was decided at, the view's, for the caller to submit with the change.

### What eight hours of chaos soak found

#### Fixed
- faction-protocol: a running node takes a membership as it was sent -- one that does not name
  the node removes it, as the specification says (§16) -- and acknowledges every membership it
  takes but one that removed it. It added itself to every membership, as a newcomer does, so a
  node behind its own removal took the coordinator's invitation to re-add it, or a member's reply
  catching it up, as a membership naming it, adopted a member no commit had made, and caught a
  restarted peer up with it. The soaks found it four times in 3.7 million seeds; each case is a
  chaos regression.
- faction-protocol: a retry or a period that re-arms -- `RetryPing`, `RetryReady`,
  `RetryMembership`, `ProbeDue` -- cancels every pending copy of itself before it schedules the
  next. A firing the runtime delivers twice, as its contract allows, left two chains, and
  repeated duplication multiplied them: a node pinged four times a round, every member answering
  each ping, and the answers' backlog kept it from answering a probe in time. The soak's case is
  a chaos regression.

#### Changed
- The chaos soak starts, when given no seed, from the wall clock's nanoseconds: it counted
  seconds, so soaks run an hour apart began 3,600 seeds apart and the second replayed nearly
  every seed the first had played. And the regressions file keeps a seed once: a soak finding a
  seed failing again records nothing new.

## [0.12.0] — 2026-11-17

### Published on crates.io

Conforms to the specification's version 0.8.0: its 115 vectors in force are kept, each re-encoded once as receipts
with no answer changed, their released bytes kept under `spec/released/0.7.0/`.

### Every answer numbered, every node an instance

#### Added
- The specification is 0.8.0: every call is answered with a receipt -- its output, the result
  it gave before, and a number one more than the instance's last, the first being 1, a rejected
  command and a probe like the rest -- and every node is an instance, its node and an
  incarnation the caller gives it, never repeating for that node. A node's log numbers its
  lines, so a gap proves a line lost. The shape is beggar's and cartel's
  (`P1-ADR-TheLogIsTheRecord`).
- `faction::instance::receipt::Receipt { number, output }`,
  `faction::instance::instance_id::InstanceId { node, incarnation }` and `Config::instance()`.

#### Changed — breaking
- `Faction::process` returns a `Receipt`, its `output` the `ProcessResult` it returned before.
- `Observer::observe` is told the command and the `Receipt`.
- `Config::new` takes an `InstanceId` in place of the peer id; `Config::peer_id()` is its node.
- faction-protocol: `Newcomer::new` and `Offer::config` take the `InstanceId`.
- The specification's vectors are re-encoded once, changing no answer: each step a command and
  its receipt, the configuration's peer id its instance's node at incarnation 1, the final
  answer the closing `Probe`'s receipt. Their released bytes are kept under
  `spec/released/0.7.0/`, and `released.json` records both digests; the growth rule allows a
  re-encoding of every released vector at once, never of an answer.

#### Changed
- The system tests' node logs carry each call's incarnation and number, and a node restarted
  or invited back is its node's next incarnation.

## [0.11.0] — 2026-11-17

### Published on crates.io

Conforms to the specification's version 0.7.0: its 115 vectors in force are released and kept, the 112 of 0.6.0
still in force pass unchanged, and vector 113, released at 0.6.0, is withdrawn to correct the defect it showed.

### A provisional node coordinates no change

#### Added
- The specification is 0.7.0: a node whose membership is provisional coordinates no change.
  Asked for an addition or a removal it answers `CoordinationRefused`, carrying the peer, and
  nothing changes, until the commit of its own addition ends the provisional membership: an
  addition it coordinated handed the newcomer a membership naming a member no commit added. One
  row added, one gone -- an acknowledgement from behind at a provisional node, which no node
  reaches any more -- and vectors 114 to 116.

#### Changed — breaking
- `Outcome` gains `CoordinationRefused { peer_id }`.
- A node whose membership is provisional refuses `AdditionRequested` and `RemovalRequested`,
  which 0.10.0 answered as any concluded node does.

#### Removed
- Vector 113, released at 0.6.0, is withdrawn: it showed a provisional node coordinating an
  addition. The specification's growth rule (§1) now allows a released vector withdrawn to
  correct a defect, never to grow; `released.json` marks it withdrawn at 0.7.0, and its bytes
  are kept in `spec/withdrawn/`.

#### Changed
- The chaos tier's layer above asks a newcomer to coordinate once added, as it asks any peer no
  change removes: refused while its membership is provisional, coordinating once its own addition
  commits.

#### Fixed
- An adopted membership's missed periods are listed in its members' order, as the specification
  says (§1, §16): a node kept them in the order of the membership it held before, so a view after
  an adoption that reordered members who had missed periods differed from the specification's.
  The property tier found it, the crate against the independent model; a unit test keeps the
  case.

## [0.10.0] — 2026-11-17

### Published on crates.io

Conforms to the specification's version 0.6.0, grown by its rule from 0.5.x: its 113 vectors are released and
kept, and the 108 of 0.5.0 pass unchanged.

### The chaos validation: what it found, fixed

#### Added
- The specification is 0.6.0, grown by its rule from 0.5.x: a newcomer holds its membership
  provisionally -- the configuration and the view state it, omitted when not -- and refuses a
  claim from behind, before or after its completion, or an acknowledgement from behind,
  without sending the membership (`StaleEpochIgnored` alone). The membership stops being
  provisional when the node applies its own addition's commit, adopts a membership, or is
  removed. Three rows, vectors 109 to 113.
- A ninth tier of the testing ladder, chaos (`validation/chaos/`): scenarios drawn from a seed --
  membership changes in partly random order under the network's and the timers' faults --
  every invariant watched while they play, and every node checked, once they heal, against the
  end state the committed log implies; on a deterministic engine whose seeds replay exactly and
  whose failures shrink into recorded regressions, and on a real one over threads and processes.
  Stage 1 runs 300 seeds and every regression; `just chaos`, `just chaos-soak` and
  `just chaos-replay <seed>` the rest.

#### Changed — breaking
- `Config::new` takes whether the membership is provisional, and `ClusterView::new` whether
  the view's is; `Config::is_provisional` and `ClusterView::is_provisional` say so.
- faction-protocol: `Offer::config` creates a newcomer's configuration provisional.
- faction-protocol: `TransportMessage::Membership` names the peer it is addressed to, `to`: a
  caller that builds or matches the message names its addressee, and a caller's codec carries it
  on the wire, or a node ignores every membership as addressed to another.

#### Fixed
- faction-protocol: a membership names the peer it is addressed to, and a node takes as
  an offer only a membership addressed to it -- a newcomer joins from it, a removed node
  is invited back by it, a running node adopts it. A membership delivered to another
  node, as a transport that misdelivers can, is ignored: a removed node no longer comes
  back on another peer's invitation, believing itself a member no one else holds.
- faction-protocol: a node answers a member still pinging it (`AcknowledgeRejoin`) by sending
  its readiness to that member alone, as the specification says, instead of broadcasting it
  to every member. Each round of a straggler's pings drew n x (n - 1) messages from n
  concluded members where n were needed, the concluded receivers ignoring theirs.

#### Changed
- The specification is 0.5.1: it answers every command as 0.5.0 does, and states what the
  layer above owes a node — its commits in epoch order. A commit reported after one at a
  higher epoch is ignored as held, and its change lost to the node, which cannot see the gap;
  a caller that cannot keep the order hands the node a membership instead.
- ARCHITECTURE states the reference protocol's timer rule — a retry re-arms until its answer
  comes, a timeout and a one-shot never re-arm — and the runtime's contract: a timer fires at
  least once.

## [0.9.0] — 2026-11-16

### Published on crates.io

Conforms to the specification's version 0.5.0, the first to grow by its own rule: its 108 vectors are released and
kept, and the 93 of 0.4.0 pass unchanged.

### Phase 5: every membership carries its epoch

#### Added
- [`spec/`](./spec/SPEC.md) 0.5.0: the epoch, the version of the membership a node holds
  (`P1-ADR-TheEpochComesWithTheCommit`), the first growth under the specification's own rule.
  The layer above numbers its changes, strictly increasing, and a commit brings its number:
  a commit at or below the node's epoch is held already, `CommitIgnored`; any other moves the
  epoch. A member's participation or readiness, or the awaited acknowledgement, from behind
  is refused, `StaleEpochIgnored`, and answered with the membership, `SendMembership`;
  liveness carries no epoch. A new command, `MembershipReceived`, admitted in every stage: at
  or below the epoch, `MembershipIgnored`; above it, adopted, `MembershipAdopted` -- the
  stage kept and its lists kept to the new members, a collecting node concluding on the new
  count, a change in flight given up, `ChangeAbandoned`, a removed node it names back to
  `Initial`, a node it does not name removed. The configuration and the view carry the epoch.
  Fourteen rows (70 in all); fifteen vectors (108 in all). Epoch 0 is omitted wherever it
  would stand, and the 93 vectors of 0.4.0 pass unchanged; `spec/released.json` lists the
  commands each released version names, and a released vector's admissible commands are
  compared among them.
- The specification's independent model answers 0.5.0 before the crate does.
- The crate: `faction::types::Epoch`, `Config::epoch`, `ClusterView::epoch`,
  `ClusterViewBuilder::from_view_at`, `faction::states::epoch_rule::EpochRule`,
  `faction::states::adoption::Adoption`, `Removed::departure`, `MembershipDecision::Adopted`,
  `Bootstrapped::changed` and `Bootstrapped::departure`.
- `faction-protocol`: the epoch on `Ping`, `Ready`, `Membership` and
  `MembershipAcknowledged`, and on the broadcasts; a running node takes a membership a peer
  sends as a newcomer does, through `faction_protocol::offer::Offer`, and acknowledges it at
  its new epoch; `Protocol::invites`, for whoever runs a removed node to make it a newcomer
  again; `Protocol::commit(Commit)`, the layer above's commit and its epoch.
- The matrix tier covers the fourteen rows; the property tier draws epochs and memberships,
  and holds that a node's epoch never decreases and that a claim from behind never counts;
  the in-process cluster restarts a node and runs five scenarios; the system tests restart
  nodes in place, re-add a removed one, partition one behind and set one ahead, in every
  spawn and transport (50 tests), the harness numbering its commits as the layer above.
- A memory tier, `validation/memory/`, as beggar's and cartel's: a counting allocator in a test binary of its own
  holds the README's claim that the machine is slim by construction. A conclusion holds more for more peers and
  nothing for the duplicates and probes between its signals; a thousand probe periods, each suspecting a member and
  reviving it, hold what one does; a hundred membership changes, each a newcomer added and removed, hold what one
  does; and nothing is left once the machine is gone.

#### Changed — breaking
- `Command` gains `MembershipReceived` and an `epoch` on six variants, and is no longer `Copy`;
  `Outcome` gains its five variants; `Config::new` takes the epoch; `ClusterView::new` takes it.
- `State::step` takes the node's epoch, and `Stage::admissible_commands` with it.
- `Removed::new` takes the membership it holds; `Removed::own_removal` takes the members and
  the count.
- `faction-protocol`: `commit(Commit)` in place of `commit_addition`, `commit_removal` and
  `change_required_count`; `OutputMessage::BroadcastPing` and `BroadcastReady` carry the
  epoch; `MessageTranslator::to_command` takes the node's epoch; `Protocol::initialize` takes
  `&mut self`.

#### Changed
- A removed node no longer retries its pings or its readiness.
- The system tests' indirect-probe scenario waits the generous probe stretch in its one case, processes over gRPC,
  that twice suspected its cut member under the gates' parallel load.

## [0.8.3] — 2026-11-16

### Published on crates.io

A patch release: nothing the crate does changes. Conforms to the specification's version 0.4.1, which answers every
command as 0.4.0 does and keeps its 93 vectors as released.

#### Changed
- The specification is 0.4.1: it answers every command as 0.4.0 does, and keeps a release's vectors as released, as
  beggar's and cartel's do. The growth rule omits a member a later version adds at its initial value and names a
  command a later version adds in no earlier vector's admissible commands; `spec/released.json` lists all 93 vectors
  by the version that released their bytes and the SHA-256 of them, and a test fails on one removed or changed.
  Before the rule the encoding grew in place -- 0.2.0 to 0.4.0 rewrote the vectors released before them, never an
  outcome -- which the specification records.
- The conformance tier checks that no public name of the crate and no name of the schema holds a word the
  vocabulary leaves out, as cartel's does: cartel's list less `view`, `commit`, `committed` and `epoch`, faction's
  own (`docs/ARCHITECTURE.md`). The gates' duplication check comes down to the floor its siblings use, 10 nodes,
  against a baseline of the nine groups already there.

## [0.8.2] — 2026-11-16

### Published on crates.io

A patch release: nothing the crate does changes. Conforms to the specification's version 0.4.0.

#### Changed
- Edition 2024, as the three bookkeepers now are; `rust-version` stays 1.85. rustfmt's 2024 style reflows the
  source. A crate's edition is its own, so no caller is touched.
- The specification's model returns its members' iterator `+ use<'a>`, as 2024's capture rule asks. The gates hand
  the system tests their run's name through the process runner rather than setting it in their own environment.

## [0.8.1] — 2026-11-16

### Published on crates.io

A patch release: nothing the crate does changes. Conforms to the specification's version 0.4.0.

#### Changed
- The crate's description and keywords name what `faction` now is, a cluster membership state machine, where they
  said readiness and startup coordination.
- The README and the crate's front page follow the outline the three bookkeepers now share; the tagline names what
  `faction` now is, a cluster membership state machine. The quick start asks for 0.8 and runs as written.
- The agent rules carry the strict rules beggar's and cartel's do: tests first, never removing a test or relaxing an
  assertion without asking, no re-export shims, and additive releases as faction keeps them.

## [0.8.0] — 2026-11-13

### Phase 4: a concluded cluster shrinks by one

#### Added
- [`spec/`](./spec/SPEC.md) 0.4.0: removal, a concluded cluster shrinking by one through the layer
  that commits changes above it (`P1-ADR-RemovalThroughTheLayerThatCommits`), coordinated as an
  addition is but with no handshake. A node refuses to coordinate the removal of a member not
  suspected when the members not suspected would number fewer than the required count without
  it; a suspect's removal it never refuses for the quorum. A commit is applied on every node, in
  every stage, the peer leaving every list the node holds, its liveness with it; it may lose the
  quorum, and never restores it. A sixth stage, `Removed` — a node whose own removal committed,
  admitting a probe alone, never shown `TimedOut`. Three commands, `RemovalRequested`,
  `RemovalCommitted` and `RemovalAborted`; four outcomes, `ProposeRemoval`, `RemovalRefused`,
  `RemovalWithdrawn` and `MemberRemoved`; the peer state `Removed`; ten rows (56 in all) and four
  widened; 15 vectors (93 in all). The 0.3.0 vectors change only in the view's field and the
  admissible lists.
- The specification's independent model answers 0.4.0 before the crate does; it caught three of
  the new vectors as first written.
- The crate: the commands, the outcomes, `PeerState::Removed`,
  `faction::states::removed::Removed`, `Members::without` and `Members::retained`, and the
  liveness forgetting a member no longer one (`Liveness::of_members`, with `Probing`'s and
  `Suspicion`'s).
- `faction-protocol`: `request_removal`, `commit_removal` and `abort_removal`, and the output
  `ProposeRemoval`; a member removed has its probe timeouts cancelled. No message on the wire.
- The matrix tier covers the sixth stage and the three commands; the property tier draws them,
  and holds that a removal is proposed exactly when the quorum can spare the member; the
  in-process cluster runs seven scenarios; the system tests remove nodes in every spawn and
  transport (50 tests), the process node's state line writing `removing:N` and `Removed`.

#### Changed — breaking
- The view's addition is its reconfiguration, one change in flight of either kind:
  `faction::reconfiguration::Reconfiguration` — `None`, `AdditionRequested`,
  `AdditionAcknowledged` and `RemovalRequested` — in place of `faction::addition::Addition`;
  `ClusterView::reconfiguration` and `ClusterViewBuilder::with_reconfiguration` in place of
  `addition` and `with_addition`.
- `Reconfiguring::new` takes the change in flight.
- `PeerState` gains `Removed`; `Command` and `Outcome` gain their variants.

#### Changed
- The ladder's invariants that held only while nothing was removed are stated exactly: the peers
  gathered shrink by the removed peer alone, and a node's own removal leaves it `Removed` for good.
- A node's own removal is taken by each stage that admits it, through `Removed::own_removal`;
  `Stage` only delegates (`P2-ADR-StateStructsBehindStageEnum`).
- `Pinging` takes readiness before its own completion through `ReadyBeforeCompletionStep`, as it
  takes participation through `PingingStep`.

## [0.7.0] — 2026-11-13

### Phase 2: a concluded node watches its members' liveness

#### Added
- [`spec/`](./spec/SPEC.md) 0.3.0: liveness, a concluded node probing its members one period at a
  time, as SWIM's failure detector does, the caller keeping the clock
  (`P1-ADR-LivenessReportedNotStaged`). The configuration gains a suspicion threshold — how many
  periods running a member must miss before it is suspected, each node's own. Four commands,
  admitted in the concluded stages alone, `ProbeDue`, `PeerHeard`, `ProbeMissed` and
  `IndirectProbesMissed`; eleven outcomes, `SendProbe`, `NothingToProbe`, `SendIndirectProbes`,
  `MissCounted`, `PeerSuspected`, `PeerRevived`, `AliveNoted`, `AlreadySuspected`, `SelfIgnored`,
  `QuorumLost` and `QuorumRestored`; seventeen rows (46 in all) — twelve of liveness and five twins
  of the admission, commit and count rows that say a crossing of the quorum; the view's
  suspected members, missed periods, member probed last and quorum; 16 vectors (78 in all). The
  0.2.0 vectors change only in their configurations, their views and the concluded stages'
  admissible lists.
- The specification's independent model answers 0.3.0 before the crate does.
- The crate: the commands, the outcomes, `faction::quorum::Quorum`, `faction::miss_count::MissCount`,
  `ClusterView`'s `suspected`, `misses`, `last_probed` and `quorum`, the builder's four `with_`
  methods, `Config::suspicion_threshold`, and `faction::states::liveness::Liveness` — made of
  `Probing` and `Suspicion` — shared by `Bootstrapped` and `Reconfiguring`.
- `faction-protocol`: the messages `Probe`, `ProbeAnswer`, `IndirectProbe` and
  `IndirectProbeAnswer`, a probe and its answer naming whom they are relayed for; the timers
  `ProbeDue`, started at the conclusion, `ProbeMissed` and `IndirectProbesMissed`; `ProbeRelay`,
  answering and relaying probes with no table of its own; only probe answers count as a sign of
  life.
- The matrix tier covers the four commands in every stage; the property tier draws them at a random
  threshold, against the model, and holds three invariants of liveness; the in-process cluster
  holds each node's liveness timers until a test elapses them, and runs ten scenarios; the system
  tests silence members and cut links in every spawn and transport (50 tests), through a
  `FaultyTransport` now around every node's transport, its faults in the node's log.

#### Changed — breaking
- `Config::new` takes the suspicion threshold.
- `ClusterView::new` takes the suspected members, the missed periods, the member probed last and
  the quorum.
- `Bootstrapped::answer` takes the configuration.
- `Protocol::new` takes the helpers a missed probe is followed up through, and
  `MessageTranslator::new` takes them too, its default asking every member listed;
  `Newcomer::new` takes its suspicion threshold and its helpers.

## [0.6.0] — 2026-11-13

### Phase 3: a concluded cluster grows by one

#### Added
- [`spec/`](./spec/SPEC.md) 0.2.0: addition, a concluded node growing by one through the layer that
  commits changes above it (`P1-ADR-AdditionThroughTheLayerThatCommits`). A fifth stage,
  `Reconfiguring` — concluded, its peer state `Bootstrapped`, its view naming the addition in
  flight; five commands, `AdditionRequested`, `MembershipAcknowledged`, `AdditionCommitted`,
  `AdditionAborted` and `RequiredCountChanged`; six outcomes, `SendMembership`, `ProposeAddition`,
  `AdditionWithdrawn`, `AcknowledgementIgnored`, `AbortIgnored` and `RequiredCountSet`; eleven
  rows (29 in all); 21 vectors (62 in all). The 0.1.0 vectors change only in their views, which
  gain the addition, and their admissible lists, which gain the new commands.
- The specification's independent model answers 0.2.0 before the crate does.
- The crate: the commands, the outcomes, `faction::addition::Addition`, `ClusterView::addition`,
  `ClusterViewBuilder::with_addition` and `faction::states::reconfiguring::Reconfiguring`. The
  required count in effect is state, starting at the configuration's; a count the collecting peers
  already reach concludes.
- `faction-protocol`: the messages `Membership` and `MembershipAcknowledged`; the outputs `Send`
  and `ProposeAddition`; the timer `RetryMembership`, re-sending the membership until it is
  acknowledged; `request_addition`, `commit_addition`, `abort_addition` and
  `change_required_count`; a running node acknowledging every membership; and `Newcomer`,
  which waits for its membership and becomes a node of it.
- The matrix tier covers all 65 `(stage × command)` pairs; the property tier and the invariants
  draw the new commands; the in-process cluster runs six addition scenarios under loss; the
  system tests add a node to a running cluster in every spawn and transport (50 tests).

#### Changed — breaking
- `faction::states::join_decision::JoinDecision` and `join_step::JoinStep` are
  `membership_decision::MembershipDecision` and `membership_step::MembershipStep`: a join
  decision, a committed addition or a new count.
- The state structs' constructors take the count in effect, a `QuorumPolicy`;
  `ClusterView::new` takes the addition.
- `MessageTranslator::to_output_messages` takes the result's view and the local peer id.

#### Fixed
- `faction-protocol`: the translator turns every outcome of a result into messages, in order;
  it stopped at the first it recognised.
- System tests: TCP and gRPC share one length-prefixed codec, and TCP keeps a receive buffer per
  connection, queuing every connection's whole messages in arrival order — one shared buffer
  worked only while every frame was the same nine bytes.

## [0.5.0] — 2026-11-13

### Laid out by role

#### Changed
- The workspace is laid out by role (`P2-ADR-WorkspaceLaidOutByRole`): `crates/` holds the library —
  `core/` (`faction`), `protocol/` and `spec-model/` — `validation/` everything that checks it —
  `ladder/`, `core-validation/`, `protocol-validation/` and `system-tests/` — beside `spec/` and
  `xtask/`. No package is renamed; a path dependency on the crate names `crates/core`. The commit
  scores' title names `crates/`.

### Specified: faction conforms to its specification 0.1.0

#### Added
- [`spec/`](./spec/SPEC.md), the normative specification, 0.1.0: a JSON Schema data model, a decision
  table — what each stage admits, and 18 rows in check order — the prose, and 41 conformance vectors,
  103 steps, each naming the rows it exercises, every row exercised. Written by hand and versioned
  apart from the crate; the crate is one implementation of it
  (`P0-ADR-SpecificationIsNormativeCrateIsReference`). Peers are listed in arrival order.
- The conformance tier in `validation/ladder/`: every vector run against the crate, each result, the final
  answer and every report to the observer as written.
- The replay tier in `validation/ladder/`: a node's log — its configuration, then every call it reported, as
  the specification writes a step — replays whole, for random command sequences and for every node of
  a simulated cluster; a log altered, cut, reordered or opened with another configuration does not.
- `P2-ADR-SpecifiedBeforeAnyNewPhase`, superseding the deferral of the specification until hardware
  validation: every later phase changes the specification first.
- `crates/spec-model/`, the specification's independent model (`faction-spec-model`, unpublished): a second
  implementation, written from the prose and the decision table, depending on nothing of the crate
  — `deny.toml` refuses it — `no_std`, free of unsafe code, and held to every gate the library is.
  It reads the decision table and answers each command, in the specification's encoding, by the first
  row that applies; its conformance tier checks every vector before the crate runs it, each deciding
  by exactly the rows it names, and the property tier checks the crate against it over random
  command sequences (`P1-ADR-IndependentModelOfTheSpecification`). Its grip and braintax join every
  commit message's scores.

#### Removed
- The property tier's own reference model, `ModelCoordinator`, written with the crate before the
  specification: the crate against the specification's model checks everything it did — each
  command's outcomes and the view after it — and more: every result whole, a rejection told apart
  from an acceptance with no outcomes, the peers listed rather than counted, and every required count.

#### Changed — breaking
- The observer is one method, `observe(&command, &result)`: every call — accepted, rejected or a
  probe — is reported once with the `ProcessResult` it was answered. `observe_query`,
  `observe_rejection` and `Transition` are gone (`P2-ADR-TotalObservability`, amended).

#### Changed
- The transition matrix names the row of the decision table each case exercises and is checked
  against it; its expected admissible lists are the table's.
- `faction-core-validation`: `ClusterSimulation::new` takes one observer per node, so a simulated
  cluster's nodes can log.

### No line the specification does not ask for

#### Added
- A second coverage gate in stage 2: `faction` at 100% of lines, regions and functions from the
  conformance vectors alone (`P1-ADR-NoLineTheSpecificationDoesNotAskFor`).
- `faction-protocol`: `ResultView::of`, the view any result carries, for the caller that needs it.

#### Changed — breaking
- Removed, as nothing the specification asks for and each a read of what it does show:
  `ClusterView::conclusion` and `is_concluded` (a node is concluded when its peer state is
  `Bootstrapped`), `Config::is_member` and `peer_count`, `Faction::config`, `Members::is_empty` and
  `len`, `ProcessResult::into_cluster_view`, and `ClusterViewBuilder::new` (use `default()`).
- `State` has no `accept`: a stage's `step` answers `None` for a command it does not admit, and the
  admissible commands are derived from it. `JoinStep::new` takes a `JoinDecision`.
- `Members::with_admitted` answers `None` for a peer that already is a member; the one check of
  membership on admission is there.

#### Changed
- `ReadyStep` and `LocalCompletionStep` ask the `QuorumPolicy` whether quorum is reached.

### Clean-up and restructure to the house rules

#### Changed — breaking
- The states are held in a `Stage` enum (`faction::stage::Stage`), one variant per
  state struct, instead of `Box<dyn State>` (`P2-ADR-StateStructsBehindStageEnum`,
  superseding `P2-ADR-StateAsTraitObject`). `State::step` returns
  `(Vec<Outcome>, Stage)`; the trait has no default bodies and no
  `admissible_commands`, which `Stage` now derives from `accept`.
- `ClusterView`'s fields are private, read through the getters it already had. It is
  built through `ClusterViewBuilder` or `ClusterView::new`, which resolves a missed
  deadline into `TimedOut` before bootstrap, so no view carries fields that disagree
  (`P2-ADR-ClusterViewBuilderAndDto`, amended).
- `faction-protocol` (workspace-only): the modules `timer_trait` and
  `transport_trait` are renamed `timer` and `transport`.

#### Added
- `just stage1` and `just stage2`, driven by an `xtask` crate: formatting, clippy over
  every target, the `no_std` check for a bare-metal target and every test tier, then
  `cargo stern4rust`, `cargo dry4rust`, `cargo crap4rust`, 100% coverage, `cargo
  twin4rust`, `cargo iceberg4rust` and `cargo deny`. The toolchain is pinned.
- `faction-ladder`, the package holding the transition matrix and the property tiers.
- `ClusterInvariants` in `faction-core-validation`: no peer counted twice, only
  members counted, no conclusion short of quorum.
- `Members::non_member_observed`, `Collecting::resolve` and
  `ProcessResult::into_cluster_view`.
- Core tests assert that a probe is observed only as a query and a rejected command
  only as a rejection.

#### Changed
- The system-test harness takes its clock, sleep and wall clock as injected ports,
  and its tests wait on the event rather than on a sleep. An in-process newcomer uses
  the cluster's freshness margin, as a process newcomer already did.

#### Removed
- `scripts/run_stage_1.ps1` and `scripts/run_stage_2.ps1`, replaced by the `just`
  recipes.
- `ScenarioEvent`, `ScenarioNode`, `ScenarioTraceEntry` and
  `ClusterSimulation::all_exited` from `faction-core-validation`, none of them used.

#### Fixed
- A gRPC process node dialled its peers at start-up and panicked if one was not yet
  listening, failing the process-gRPC system tests on about two runs in three. It now
  dials on first send.
- The harness `RealTimer` fires timers that fall due at the same instant in the order
  they were scheduled.
- Dependency advisories: `h2` and `tokio` updated past theirs, and the yanked `spin`
  release replaced.

## [0.4.0] — 2026-07-28

### Phase 1 — dynamic joining across the full spawn/transport matrix

#### Added
- Dynamic joining. The `JoinRequested` / `JoinApproved` / `JoinRejected` commands
  drive a membership axis orthogonal to the `Initial → Pinging → Collecting →
  Bootstrapped` progression, so a node in any state — including terminal
  `Bootstrapped` and non-terminal `TimedOut` — admits or denies a newcomer and keeps
  converging. Live `Members` are carried in each `State` (`Config` is the immutable
  genesis; membership is genesis ∪ admitted). Admission is a local control-plane
  decision, not a wire message.
- Natural timeout. A `DeadlineExpired` timer is scheduled at start (from
  `--freshness-margin`), so a node that never reaches quorum times out on its own,
  not only via an injected expiry.
- `ClusterView` split into a pure DTO (public fields, no mutation methods) plus a
  `ClusterViewBuilder`, and now exposes live `members`.
- System-test join suite — 61 tests: six scenarios (join-then-converge,
  join-before-bootstrap, rejected, duplicate, concurrent multi-join,
  join-after-timeout) plus a natural-timeout scenario, across Task/Thread ×
  {in-memory, channels, TCP, gRPC} and Process × {TCP, gRPC}. The harness gained
  late-join for every transport, a Thread control plane (an mpsc command queue with
  live snapshots) and a Process control plane (an stdin/stdout line protocol).
- ADRs: `P2-ADR-ClusterViewBuilderAndDto`, `P2-ADR-TestingLadder`, and
  `P1-ADR-CollectingIsNotASink`.
- `FaultyTransport` — a harness `Transport` decorator that injects seeded transport
  faults (loss, duplication, delay, reorder, partition, asymmetric, selective) behind a
  loud per-fault banner, as the instrument for the Phase-2 hardening assessment.

#### Changed
- The stage-1 gate runs the system tests as bounded-parallel per-test processes via
  [`slotgate`](https://crates.io/crates/slotgate) — a disjoint port range per slot —
  instead of a serial `--test-threads=1` pass; the gate dropped from ~3 min to ~50 s.
- License headers corrected from Apache-2.0 to the MIT SPDX one-liner across the
  workspace.
- System-test logs restructured for scale: one timestamped run folder under `logs/`,
  one subfolder per test (`<scenario>-<spawn>-<transport>-q<n>/`) holding a per-node
  `node-<id>.jsonl` and a merged `consolidated.jsonl`; the gate writes a per-run
  `summary.json` (failures first) and prunes to the newest `-MaxLogRuns` folders.
  slotgate's per-job stdout/stderr capture nests under the same run folder.
- Process nodes derive their listen port from slotgate's per-slot `PORT_RANGE`
  (distinct per node) instead of `bind(":0")`-drop-rebind.
- Harness restructure: `ProcessSpec` / `spawn_process_node` extracted into
  `process_spawn`; the `cluster` module split one-struct-per-file into `cluster.rs`
  plus a `join/` module; `wait_for_tcp_ready`, `allocate_port_addr`, and `node_writer`
  folded into `ClusterBuilder`.

#### Fixed
- The process-TCP harness transport could silently, permanently strand a node one
  message short under gate parallelism — its accept thread died on a transient
  `accept()` error, and `send()` never replaced a dead outbound stream — timing out the
  quorum-5 convergence and sub-quorum-recovery join tests on ~15% of runs. The accept
  loop now survives transient errors and `send` drops the dead stream, reconnects, and
  retries; validated across 30 consecutive clean gate runs.

### Self-describing entry point, Architecture Decision Records, and a documentation sweep

#### Added
- Architecture Decision Records under `docs/ADRs/` — nine records documenting the
  core invariants, one property per file: protocol-agnostic, pure-Mealy (no I/O),
  deterministic and replayable, `no_std` + zero-unsafe, single entry point,
  totality via the exhaustive `(state, command)` matrix, stateful-but-persistency-free,
  total observability, and state-as-trait-object.
- The README quick-start is now compiled and executed as a doctest (via
  `include_str!`), so it can no longer drift from the public API.
- The valid transition matrix now asserts the returned `cluster_view` equals a
  subsequent `Probe` for every `(state, command)` case.

#### Changed
- Upgraded `#![deny(unsafe_code)]` to `#![forbid(unsafe_code)]` at all six
  crate roots. `deny` could still be lifted locally with a reviewed
  `#[allow(unsafe_code)]`; `forbid` cannot, so the zero-unsafe guarantee now
  holds absolutely rather than by convention. `docs/ADRs/P0-ADR-NoStdZeroUnsafe.md`,
  `README.md`, `ROADMAP.md`, and the `docs/Applications/` pitches updated to
  match.
- **Breaking** — `ProcessResult::Accepted` now carries an `admissible` field:
  `Accepted { cluster_view, admissible, outcomes }`. All three result variants
  (`Accepted`, `Rejected`, `Probed`) now expose the set of next-admissible
  commands, so a caller can ask "what may I send next?" uniformly regardless of
  outcome.
- **Breaking** — removed the `faction::PeerId` re-export. Import
  `faction::types::PeerId` directly; the alias now lives in exactly one place.
- Documentation sweep across `README.md`, `ARCHITECTURE.md`, and `ROADMAP.md`:
  reconciled test counts (277) and core productive LOC (~885), propagated the
  `admissible`-on-every-variant change, softened the formal-verification claim,
  linked the new ADRs, and corrected the worked examples.
- Curated `OPEN_POINTS.md` down to the genuinely open items — the quorum-change
  question and the deferred ADRs — dropping everything now implemented, fixed,
  or captured in an ADR.
- Split the transition-matrix test helpers into focused `builder` and
  `assertions` modules.
- **Breaking** — removed the `TimedOut` state and `Conclusion::TimedOut`.
  `Bootstrapped` is now the only terminal state; a missed deadline is surfaced
  as `PeerState::TimedOut` (a derived view flag) without concluding. Added
  `Outcome::AcknowledgeRejoin { peer_id }` and
  `Outcome::DeadlineMissed { confirmed_count }`, and `Bootstrapped` now admits
  `ParticipationObserved`.

#### Fixed
- Single-node clusters (`size == 1`) now bootstrap. `Initial` accepted only
  peer-driven input, so a lone node's self-driven `LocalParticipationCompleted`
  was rejected and it never left `Initial`. `Initial` now accepts
  `LocalParticipationCompleted`, which also closes a latent ordering hazard
  where the first node to complete locally — before observing any peer — could
  wedge at any cluster size.
- **The concluded-node silent-sink bug.** A node that
  reached a terminal state stopped helping peers: `Bootstrapped` cancelled its
  retries and went silent, stranding a peer that missed its readiness; and
  `DeadlineExpired` drove the node into a `TimedOut` dead-end it could never
  leave, even when the readies it needed arrived late. Now a bootstrapped node
  re-advertises its readiness to a still-pinging peer (`AcknowledgeRejoin`), and
  a missed deadline is a non-terminal `DeadlineMissed` fact — the node stays
  receptive and still converges.
- Corrected a `protocol-validation` harness bug: `Cluster::step_transport_node`
  attributed a node's transport-triggered outputs to the incoming message's
  sender instead of the processing node.

#### Removed
- Stale `VALIDATION.md` — superseded by the ADRs and the live test suite.

---

## [0.3.3] — 2026-05-09

### Documentation & polish — runnable example, bootstrapping-gap note, import cleanup

#### Added
- Module-level documentation with a runnable example in `lib.rs` — a full
  drive of the machine (participation, duplicate rejection, `Probe`, quorum
  exit) that runs as a doctest.
- README paragraph documenting the IBFT/Raft bootstrapping gap `faction`
  fills, with downstream-integration context.

#### Changed
- Documentation refresh across the crate docs and README.
- Replaced inline module-path qualifiers (`mpsc::`, `oneshot::`, and other
  module qualifiers) with `use` imports.
- `ClusterBuilder::new`'s second parameter renamed `required` → `node_required`
  for clarity — `system-tests` harness only, not part of the published core API.

---

## [0.3.2] — 2026-05-03

### `Observer` no longer requires `Send`

#### Changed
- Removed the `: Send` supertrait bound from `faction::Observer`.
  The bound was unnecessarily strict for `no_std` and single-threaded embedded
  contexts, where thread-safety guarantees are meaningless and preventing
  non-`Send` observers (e.g. `Rc<RefCell<_>>`-backed) from being used.
  Implementations that genuinely need `Send` (e.g. `SharedFileObserver`,
  `NoOpObserver`) remain `Send` — the bound is simply no longer enforced
  at the trait level.

#### Adapted — `system-tests`
- `Node::Task` now holds `Rc<RefCell<FactionNode>>` instead of
  `Arc<Mutex<FactionNode>>` — single-threaded polling never needed the
  overhead of a mutex or the `Send` requirement.
- `Node::Thread` no longer holds a shared reference to `FactionNode`.
  The node is now constructed *inside* the spawned thread closure, which
  keeps the non-`Send` observer local to the thread that owns it.
  The final `PeerState` is communicated back to the polling loop via a
  lightweight `Arc<Mutex<PeerState>>` written once on completion.
- `Node::spawn_thread` signature changed from `(Arc<Mutex<FactionNode>>)`
  to `(impl FnOnce() -> FactionNode + Send + 'static)` — the closure
  carries only `Send` construction data; the node itself is never moved
  across thread boundaries.

---


## [0.3.1] — 2026-05-03

### Post-hardening cleanup — dead outcomes removed, unsafe denied, timer architecture unified

#### Removed
- `StaleParticipationIgnored` and `StaleReadyIgnored` from `Outcome` — these variants were
  never produced by any state. Removed to keep the public API free of dead arms.
- `FreshnessPolicy` and `FreshnessClassification` source files — orphaned files that were
  never compiled (not listed in `lib.rs`) and referenced a non-existent `Freshness` type.
- `InMemoryTimer` from `system-tests` — functionally identical to `RealTimer` after the
  `with_delay` addition; the deterministic zero-delay use case belongs to `protocol-validation`.
- `TimerKind` enum from `system-tests` — redundant once `InMemoryTimer` was removed.
- `--timer` CLI argument from `faction-node` binary — only one timer implementation remains.
- 5 convergence test variants using `InMemoryTimer` (`task_inmemory_inmemory`,
  `task_inmemory_channels`, `thread_inmemory_inmemory`, `thread_inmemory_channels`,
  `thread_inmemory_tcp`) — coverage preserved by the equivalent `real` timer variants.
  Convergence matrix reduced from 15 to 10 variants.

#### Added
- `#![deny(unsafe_code)]` at crate root in all six entry points: `faction` (core),
  `faction-core-validation`, `faction-protocol`, `faction-protocol-validation`,
  `faction-system-tests`, and `faction-node` binary. The "0-unsafe" property is now
  enforced at compile time, not just claimed in documentation.

#### Fixed
- `thread_inmemory_tcp` instability — the `InMemoryTimer` fired at zero delay, flooding
  TCP with `RetryPing`/`RetryReady` broadcasts faster than the transport could drain them.
  Root cause eliminated by removing the combination entirely.

---

## [0.3.0] — 2026-05-02

### Phase 0 hardened — Freshness removed, step structs split, transport Drop

#### Removed
- **`FreshnessPolicy`** and `FreshnessClassification` — stale/delayed/timely distinction
  removed. Every signal from a member is accepted; duplicates are deduplicated on peer_id.
  The `freshness` and `current_marker` fields removed from `Command::ParticipationObserved`
  and `Command::ReadyObserved`.
- `ObservedKind` enum — replaced by dedicated step structs.
- `ObservedOutput` wrapper — outcome logic inlined into `ObservedStep`.
- 42 stale/delayed-specific tests (193 → 145 in core).

#### Added
- `PingingStep`, `ReadyStep` (renamed from `CollectingStep`), `LocalCompletionStep`
  — three focused step structs replacing `ObservedStep::new()` and `new_local()`.
- `compute_new_state` private helpers on `Collecting` and `Pinging` — decoupled
  from step structs, taking `(is_quorum, confirmed_peers)`.
- `Drop` implementations for all four transports: `GrpcTransport` (oneshot server
  shutdown), `TcpTransport` (AtomicBool + thread join), `ChannelsTransport`,
  `InMemoryTransport`.
- gRPC transport: replaced `block_on` in `send()` with async `mpsc` channel.
  Process test went from ~15s to ~1s.
- Timer integration tests — 15 tests for `InMemoryTimer` and `RealTimer`.
- Transport integration tests — 22 tests covering all four transports.
- `SharedFileObserver` integration tests — 9 tests with `TempFile` cleanup guard.
- `"decisions"` log field now shows full list, not just count.
- `Config::new()` — 3 args (removed `freshness_policy`).
- `build_clients` in gRPC — no `block_on` on `AsyncMutex`, builds `HashMap` first.
- `spawn_server` extraction — shared between `build_server` and `new_mesh`.

#### Changed
- `Exited` → `Concluded`, `ExitMode` → `Conclusion`.
- `CollectingStep` → `ReadyStep` with required `quorum_threshold: usize` (no `Option`).
- `ReadyObserved` in `Pinging` state inlined — no step struct call.
- `start_decisions()` → `initialize()`.
- Tokio runtime: `worker_threads(2)` → `available_parallelism()`.
- `ObservedStep` → removed entirely.
- `compute_output.rs` → inlined into `observed_step.rs`, then split into three files.

#### Fixed
- Port rebinding race in gRPC `new_mesh`.
- `Collecting::new()` dead code removed.
- Temp file collision in `SharedFileObserver` tests.

---

## [0.2.0] — 2025-04-26

### Phase 0 complete — Hardened machine

The crate has been fully hardened through adversarial testing, complete matrix coverage, and a structural redesign that eliminates dead code and clarifies state ownership.

#### Added
- `StateSnapshot` trait — each state returns a delta over the previous snapshot.
  States own only their active data; frozen fields are inherited automatically.
- `ClusterSimulation` — multi-node broadcast bus for convergence testing.
- 10 observer tests covering all transition paths.
- `CHANGELOG.md`.

#### Changed
- **Renamed `Vibe` → `Machine`** across the entire workspace (50+ files).
- `deal()` → `accept()`, `punch()` → `step()`, `vibe_check()` → `snapshot()`.
- Terminal state `step()` replaced with `unreachable!()` — gated by `accept()` returning `false`.
- Bumped version to `0.2.0`.

#### Removed
- Dead `punch` match arms in `Collecting`, `ReadyByQuorum`, `ReadyByDeadline`.

---

## [0.1.0] — Initial release

- Two-phase cluster readiness state machine.
- Five states: `Initial`, `Pinging`, `Collecting`, `ReadyByQuorum`, `ReadyByDeadline`.
- Freshness classification (timely, delayed-within-margin, stale).
- Quorum-based and deadline-based exit.
- `NoOpVibeObserver` and `MachineObserver` trait.
- `faction-validation` crate with `VibeScenarioHarness`.
- Property-based invariants via `proptest`.
- 139 core tests, 29 validation tests.
