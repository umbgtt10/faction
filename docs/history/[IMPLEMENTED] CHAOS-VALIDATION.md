# [IMPLEMENTED] Chaos validation — seeded scenarios, an expected end state

**Status:** implemented on the branch `chaos`, 2026-11-16, merged 2026-11-17 (`419aa88`), every decision of §11 as
accepted at the owner's "Deal" — two engines, the deterministic one primary; the layer above committing one change at
a time, in epoch order; the timer rule and at-least-once firing; the deterministic single run in stage 1, the rest on
demand; the commit order in the specification's 0.5.1; and Embassy's faults as mapped in §5 — and the plan as
accepted, but for these departures, each found while building:

- **What it found was fixed, and the crate is released after all.** §6 kept every finding a decision for the owner;
  three were the protocol's, each reported first and fixed at the owner's word in a commit of its own, so the crate
  goes out as 0.10.0. A membership names the peer it is addressed to, and a node takes as an offer only one addressed
  to it: a removed node a misdelivered membership reached came back, believing itself a member no one held
  (`4fb34ac`). A newcomer holds its membership provisionally until its own addition commits, and refuses a claim from
  behind without sending it: one spread a member the log never committed (the specification's 0.6.0, `536ce73`). A
  member still pinging is answered alone: the broadcast drew n × (n − 1) readiness messages where n were needed
  (`c2a28b4`).
- **The harness had findings of its own, each fixed with its test.** A node stepped idle whenever the side its turn
  named was empty, so a thread's or a process's node read one message a delay, and eight of them fell seconds behind
  and missed nearly every probe; a node's lifetime, thirty seconds, stopped process nodes in the middle of a run, and
  is now its cluster's to give; a step's log line was no JSON; a timer side one window killed was revived by a later
  window; a restart at the chaos's end was skipped; a newcomer never added was expected a member; and seeds 2k and
  2k + 1 drew the same scenario.
- **The real engine plays six cases, not ten.** Task nodes are the deterministic engine's; the real one runs thread
  nodes over the four transports and process nodes over TCP and gRPC — three seeds each, eighteen runs, some seven
  minutes, under `just chaos`.
- **Restarts and a dead timer side fall only on genesis members no change removes,** so a newcomer never crashes.
  Network faults fall on any node: a newcomer mute or deaf still wedges a coordinator under head-of-line blocking,
  the shape of §5's first threat; and a clock runs twenty-one times slow or twice fast, its second. No soak failed
  for either.
- **The four documenting tests of §7 were not written.** Commits reordered, a change lost, was shown once, by
  reordering the layer above's deliveries by hand, which the oracle caught. A lost `LocalParticipationCompleted`, a
  timer side that never wakes and a fast clock are drawn as faults quiescence heals, checked for safety while they
  last; no test documents the stall each leaves when nothing heals it.
- **A run is silent, its logs on replay.** At the owner's word a run announces no fault; a failing seed is replayed
  with every node's log, `just chaos-replay <seed>`. `logs/system/` and `logs/chaos/` keep the newest fifty runs
  each, a folder a run, named for its start.
- **The single run is 300 seeds,** ten cases of thirty, some four seconds in stage 1, with every regression
  recorded: eight, all passing.

**Releases:** the crate's 0.10.0, for what the suite found. The specification's 0.5.1 states what the layer above
owes a node, its commits in epoch order, its answers unchanged; its 0.6.0 grows by the rule, a newcomer's
provisional membership.
**Order:** after Phase 5, before Phase 6. Phase 6 is where interleavings start to matter, and this suite is how we
learn what Phase 6 must guarantee.

The suite answers: *"Whatever the network and the timers do, and in whatever order the changes land, does every
node end where the committed changes say it must — and is it safe along the way?"*

---

## 1. What it adds

A validation package, `validation/chaos/` (`faction-chaos`), that:

- generates scenarios from a seed: a cluster, a set of membership changes, network faults and timer faults;
- drives several changes at once, from different coordinators, in partly random order;
- defines a scenario by its initial state and its expected final state, the final state computed, never observed;
- checks safety while the scenario runs, and the end state once every fault has healed;
- runs once — a fixed set of seeds — or in a loop for hours, new seeds each round;
- builds on `validation/system-tests`: its cluster, nodes, recipes, transports and fault layer.

## 2. Where it starts

From a reading of the code at `a16bf3e`:

- **A seeded fault layer exists.** `FaultPolicy` draws loss, duplication, delay, reordering, partitions, asymmetric
  links and the loss of one kind of message, from a seed; `FaultyTransport` applies it to any transport, and
  `LinkCuts` cuts and mends links while a cluster runs. Every system test uses `FaultPolicy::default()`, no fault.
- **Timers are a trait.** `Timer` (`poll`, `schedule`, `cancel`) is the protocol's seam; `RealTimer` runs on an
  injectable `MonotonicClock`, and the tests hold fake clocks and pauses that advance on their own.
- **The harness already plays the layer above.** It numbers its commits, restarts a node in place from genesis or a
  snapshot, commits a change some nodes miss and catches them up later (`Cluster::catch_up`), and makes a removed node
  invited back a newcomer.
- **Polls never give up.** `Cluster::poll_until` loops until its condition holds; a run that never converges hangs,
  and only slotgate's per-test timeout ends it.
- **Each node takes one change in flight; the cluster takes several.** Two coordinators may each propose a change;
  only the layer above decides what commits.
- **Embassy, read for what it can produce** (`etheram-embassy`'s timer infrastructure, its contract apps and their
  verdicts, the QEMU clock finding), at the owner's word: mirroring it is not required, simulating what it can do is
  (§5).
- **Nothing states the order commits reach a node in.** A node applies commit 5, then ignores commit 4 as held —
  and has missed change 4 for good. Epochs need not be contiguous (an index may jump), so a node cannot see the gap.

## 3. The design

**One scenario format, two engines.**

| | Deterministic engine | Real engine |
|---|---|---|
| Nodes | task, one thread | thread and process |
| Transport | in-memory, through `FaultyTransport` | in-memory, channels, TCP, gRPC, through `FaultyTransport` |
| Time | a simulated clock the engine advances | the machine's clock, `FaultyTimer` over `RealTimer` |
| A seed fixes | the scenario and every interleaving: a run replays exactly | the scenario, not the scheduling |
| Speed | thousands of scenarios a minute | seconds to minutes each |
| Finds | logic: orders, faults, races the model allows | sockets, threads, the process boundary, real timing |

**A scenario** is a seed, a cluster (size, required count, suspicion threshold), a timeline of steps (changes,
faults, heals, restarts) and a deadline. Its expected end state is computed from its initial state and the log the
layer above commits. A failing scenario is written out whole, as JSON, beside its seed.

**A run** has three phases: chaos, where the timeline plays under faults; quiescence, where every link mends, every
timer becomes faithful and the layer above catches every node up; and the verdict, the end state checked before the
deadline.

## 4. Decisions

**D1. Two engines, one scenario.** The deterministic engine is the primary one: it replays a seed exactly, shrinks a
failure, and loops for hours. The real engine runs the same scenarios where a seed fixes the scenario but not the
interleaving; a failure there keeps its seed and the node logs. *Rejected:* the real engine alone — a failure it
finds may never recur.

**D2. The harness is the layer above, and it commits one change at a time.** It orders the proposals it receives,
numbers them, and commits them; a proposal it cannot commit — one the committed membership has made moot, or a second
change while one is pending — it aborts, as a consensus layer would. Commits reach each node in epoch order, possibly
late, possibly duplicated, never reordered. One test shows what reordering does: a change lost. Concurrency is in the
proposals, the faults and the delivery, never in two conflicting commits — the latter is Phase 6's question, which the
suite will take up when Phase 6 does.

**D3. The expected end state is computed.** From the initial membership and the committed log: every live member
holds the members, the required count and the epoch of the last commit, is `Bootstrapped`, holds its quorum, has no
change in flight and suspects no one; a node the log removed is `Removed`; one it took back is a member. The verdict
compares each node's view to it once quiescence has run. *Rejected:* an end state observed from a fault-free run — it
would test faction against itself.

**D4. Safety is checked while the scenario runs.** Through each node's observer, after every call:

- the epoch never decreases;
- a claim from behind never counts;
- a node concludes only with the required count of collecting peers;
- a node never shows a peer it was never told of as a member;
- a removed node answers nothing but a membership and a probe.

An end state alone would miss a stale node counted for a moment and then forgotten.

**D5. No expected outcome depends on timing.** A change refused for the quorum — which depends on whom the
coordinator suspects, which depends on timing — is retried by the layer above until it commits once the network has
healed, or the generator leaves it out. A run's verdict is therefore exact: one end state, never a set of possible
ones.

**D6. The network faults.** What `FaultPolicy` draws — loss, duplication, delay, reordering, partitions, asymmetric
links, one kind of message lost — plus the timeline's own: a partition that heals, a node silenced and heard again, a
node crashed and restarted from genesis or from a snapshot, the layer above reaching a node late. And what Embassy can
produce (§5): head-of-line blocking, a bounded inbox dropping its tail under a burst, loss growing with a message's
size at a busy receiver, a node mute or deaf until it restarts, a datagram delivered to the wrong node, the first
messages to a peer lost. Corruption stays out: the codec turns garbage into a loss, and a lying peer is the Notary's.

**D7. The timer faults, and the rule they test.** The protocol's timer rule, already true and now written down:

| Timer | Kind | On firing |
|---|---|---|
| `RetryPing`, `RetryReady` | retry | re-broadcasts and re-arms, until the node concludes or is removed |
| `RetryMembership` | retry | re-sends the membership and re-arms, until the newcomer acknowledges |
| `ProbeDue` | period | probes the next member and re-arms, while the node is concluded |
| `ProbeMissed`, `IndirectProbesMissed` | timeout | no re-arm: its firing is the verdict; an answer cancels it |
| `DeadlineExpired` | timeout | no re-arm: it records a fact |
| `LocalParticipationCompleted`, `ParticipationObserved` | one-shot | no re-arm: fired once, at start |

The runtime's contract: **a scheduled timer fires at least once** — possibly late, early within a bound, twice, out
of order with another, or after a cancel lost its race. `FaultyTimer`, over any `Timer` as `FaultyTransport` is over
any transport, draws each of these from the seed, and skew and drift between nodes, and the starvation of a node's
timer side or its transport side — and what Embassy can produce (§5): a timer side that never wakes again until the
node restarts, a delivery that fails and is retried later, a pending timer replaced by its own reschedule, a clock at
a wrong rate. Under the contract the end state is exact. A timer lost for good breaks the
contract: its scenarios check safety alone, and one test documents the stall — a lost
`LocalParticipationCompleted`, which leaves its node never concluding. Surviving a lost timer is a protocol step of
its own, if the owner wants it (§10).

**D8. The changes.** Additions, removals, count changes, the re-addition of a removed node, restarts from genesis and
from a snapshot, aborts — proposed on several coordinators at once, in an order the seed draws, the layer above
serialising what commits (D2). The generator keeps the committed log valid: the required count never above the
members, a removal never of the last member.

**D9. Run once, or for hours.** Once: a fixed list of seeds in the deterministic engine — fast enough for stage 1 —
and a shorter list in the real engine, under `just chaos`. For hours: `just chaos-soak`, with `CHAOS_DURATION` or
`CHAOS_SCENARIOS`, new seeds each round, a summary as it goes. A failing seed is shrunk in the deterministic engine —
fewer changes, fewer faults, a shorter timeline — and written to `chaos-regressions.json`, which every single run
replays from then on, as proptest's regressions are.

**D10. Where it lives.** `validation/chaos/`, a package depending on `faction-system-tests` for its cluster, nodes,
recipes, roles, transports and fault layer. `system-tests` gains what both need:

- polls that give up at a deadline and say what they waited for;
- `FaultyTimer`, beside `FaultyTransport`: late, early, duplicated and reordered firings, lost cancels, a timer side
  that never wakes again, delayed delivery, a reschedule that replaces or adds, a clock's rate, drift and grain, and
  starvation;
- `FaultyTransport` widened by §5's rows: head-of-line blocking, a bounded inbox, loss by size and by burst, a mute
  or a deaf node, misdelivery, the first contact lost;
- a fault policy that can change while a cluster runs;
- the simulated clock, from the tests' fakes into the library;
- an observer that checks the invariants of D4 and reports the first breach.

The deterministic engine steps task nodes one at a time on that clock; the real engine is `Cluster` as it is, with
faults switched on.

**D11. The specification 0.5.1 states the commit order.** §16 gains one sentence: the layer above reports its commits
to a node in epoch order; a commit it reports late is applied, one it reports twice ignored, and one reported out of
order is outside the specification. No answer changes, so no vector does. ARCHITECTURE records the timer rule and the
runtime's contract of D7.

## 5. What Embassy can produce, and how the suite produces it

Read in `etheram-embassy`, not mirrored: each row is a behaviour its boards or its QEMU runs can show, the fault the
suite draws for it, and when that fault heals. A fault that heals only by a restart is healed in quiescence by one.

| What Embassy does | Seen in | The suite's fault | In the contract? | Healed by |
|---|---|---|---|---|
| An undeliverable datagram — a peer that never answers ARP — holds the transmit ring, and nothing behind it is sent, to anyone | `tx-wedge-app` | head-of-line blocking: one unreachable destination stalls a node's sends to every peer | yes | the destination reachable again, or a restart |
| A receive ring of two datagrams, drained by a busy consumer, drops the third | `fragment-loss-app`: 17% lost at two fragments, 67% at three | a bounded inbox dropping its tail; loss growing with a message's size, at the receiver with a backlog | yes | the burst ending |
| Ethernet frames dropped under a flood, and not at rest | `eth-drop-contract-app` | loss rising with the burst — a broadcast storm — and falling when it ends | yes | the burst ending |
| A socket's transmit path wedged while the node still logs and receives | the cluster sweeps behind `tx-wedge-app` | a mute node: it receives, and sends nothing | yes, as a partition | a restart |
| A lost waker on the network task | the alarm table, below | a deaf node: it sends, and receives nothing | yes, as a partition | a restart |
| Two boards answering ARP for one address; the traffic reaches whichever answered last | the contract apps' addressing notes | a datagram delivered to a node it was not sent to | yes | the run's end |
| The first datagrams to a peer waiting on, or lost to, neighbour resolution | `tx-wedge-app`, the 60 s neighbour cache | the first messages to each peer lost or late | yes | contact made |
| An alarm table at capacity evicts the furthest-out waker or drops the new one; a task whose waker is lost never runs again | `alarm_table.rs`, `alarm_diagnostics.rs`, `RunInvariants` | a timer side that never fires again on its node | **no**: safety alone until healed | a restart |
| An expired timer whose delivery fails stays pending and is retried | `timer_dimension_state.rs` | a firing delivered late, after later inputs | yes | — |
| A timer rescheduled under its own key replaces the pending one | `timer_dimension_state.rs` | a reschedule that postpones a pending firing, as the runtime chooses: replace or add | yes | — |
| embassy-time 21× slow under QEMU, from a SysTick reload for the wrong core clock | the QEMU clock fix, 2026-10-19 | a node's clock at a gross rate — slow or fast | **no**: a fast clock times out answers before they can come | the clock restored |
| A tick 272.7 ppm fast from a truncating prescaler; an oscillator's drift; ticks of 15 ppm grain | `timer_contract_verdict.rs`, `tick_rate.rs`, `drift_estimate.rs` | skew and drift between nodes, and coarse ticks | yes | — |
| A `wfi` loop starving the network task, so what the sockets hold is never sent | the contract apps' notes | the transport side starved while the timer side runs, and the reverse | yes, within a bound | the starvation ending |
| A watchdog reset, a panic, a brown-out; persistence in RTC backup registers | crash recovery in its ARCHITECTURE | a crash, restarted from genesis or from a snapshot — a stale one among them | yes | the restart itself |
| Storage pressure stalling a node | IBFT's scenario 17 | a slow node: every step delayed | yes | the pressure ending |

Two of these threaten faction as it stands, and the first soak will say how much:

- **A coordinator retrying a membership to a newcomer that died wedges its own sends**, under head-of-line blocking:
  it goes mute to the cluster, which suspects a healthy member. The quorum may be lost for a while; the end state, once
  healed, must still hold.
- **A node with a fast clock suspects members that answer**, its timeouts expiring before the answers can arrive. This
  is outside the contract — safety alone while it lasts — and it heals when the clock does.

## 6. Where faction stands, not changed by this proposal

The suite tests faction 0.9.0 as it is. What it finds is reported, not fixed, in the suite's own commits: a finding is
a decision for the owner — a fix, a statement in the specification, or a limit written down.

## 7. The system and scenario counts

| Tier | What | Count |
|---|---|---|
| Deterministic, once | fixed seeds, every scenario shape | a few hundred, under a minute |
| Real, once | fixed seeds, the ten spawn and transport cases | a few dozen, minutes |
| Soak | new seeds, either engine | as long as it is asked to run |
| Documenting tests | commits reordered; a lost `LocalParticipationCompleted`; a timer side that never wakes; a fast clock | 4 |

## 8. Order of work

1. This proposal, agreed.
2. The specification 0.5.1: the commit order. The timer rule and contract in ARCHITECTURE.
3. `system-tests`: polls with deadlines, `FaultyTimer`, a live fault policy, the simulated clock, the invariant
   observer — each with its tests.
4. `faction-chaos`: the scenario, its generator, the layer above, the oracle, the deterministic engine; the single
   run's seeds.
5. The real engine; `just chaos` and `just chaos-soak`; the regressions file and shrinking.
6. A first soak, its findings reported to the owner.
7. The docs: the testing-ladder ADR gains the tier, the README its row, the ROADMAP the step.

Each step on a branch, every gate green on every commit.

## 9. Out of scope

- Conflicting commits, and changes queued in faction — Phase 6's.
- A lying peer, corruption — the Notary's.
- Performance and throughput — faction cares about delivery, not speed.
- The Embassy deployments — their own repositories.

## 10. Later, if the owner wants it

- **Surviving a lost timer.** The protocol's one-shots made retries, and its chains re-armed from any input that finds
  them absent.
- **The suite in the gates.** Its single run in stage 1, once its time is known.

## 11. For the owner to decide

1. **D1:** two engines, the deterministic one primary.
2. **D2:** the layer above commits one change at a time, in epoch order; conflicting commits wait for Phase 6.
3. **D7:** the timer rule as written, and at-least-once firing as the runtime's contract; a timer lost for good
   checked for safety alone.
4. **D9:** the deterministic single run in stage 1, the real one and the soak on demand.
5. **D11:** the commit order stated in the specification, as 0.5.1.
6. **§5:** Embassy's faults as mapped; the two outside the contract — a timer side that never wakes, a clock at a
   gross rate — checked for safety alone until quiescence heals them, by a restart and a restored clock.
