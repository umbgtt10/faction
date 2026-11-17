# [IMPLEMENTED] Numbered receipts — every answer numbered, every instance named

**Status:** accepted 2026-11-17, at the owner's answers: the receipt's answer named `output`, as the siblings name it;
the instance id the siblings' `InstanceId { node, incarnation }`, the caller's to give; the vectors written as the
siblings write theirs, every released vector re-encoded once with no answer changed; a log's lines numbered as the
siblings number theirs. Implemented on the branch `numbered-receipts`, 2026-11-17, as accepted, but for these
departures, each found while building:

- **The two records live in a folder of their own:** `faction::instance::instance_id::InstanceId` and
  `faction::instance::receipt::Receipt`, where §5 named them at the crate's root, which holds the twenty files a
  directory may.
- **The conformance tier reads every vector's configuration into the crate and back.** The crate itself never reads
  the instance back, so `Config::instance()` was a line the conformance tier alone left uncovered; read back, the
  instance a configuration states is one the crate holds (`P1-ADR-NoLineTheSpecificationDoesNotAskFor`).
- **The re-encoding is checked twice:** a script wrote it, and the conformance tier derives every re-encoded vector
  from its released bytes on its own and compares.
- **A log's line in the system tests carries the incarnation too**, beside the number: a node restarted keeps
  writing to its file, and its numbers starting again are told from a gap by it. The in-process cluster of
  protocol-validation gives a restarted or rejoined node its next incarnation, as the system tests do.
- **A probe cut from a log** is the evidence: the replay tier finds it by the gap in the numbers, where before nothing
  showed.

**Releases:** the specification's 0.8.0; the crate's 0.12.0, breaking.
**Order:** after 0.11.0, before Phase 6, at the owner's word.

The change answers: *"Is a node's log whole — and whose is it?"*

---

## 1. What it adds

- Every call is answered with a **receipt**: an **output** — the result it gives today, `Accepted`, `Rejected` or
  `Probed` — and a **number**, one more than the instance's last, the first being 1, whatever the output.
- Every node is an **instance**, named by an **instance id**: its node's id and an **incarnation** the caller gives
  it. A node created again — restarted, or a removed node invited back — is a new instance, numbering from 1.
- A log's lines carry their numbers, so a gap proves a line lost, and the instance id tells two runs' logs apart.

This is the one difference of shape the 2026-11-16 consistency check found between faction and its siblings, beggar
and cartel, that is not faction's state machine (`docs/OPEN_POINTS.md`, "Numbered results"). Faction takes the
siblings' shape.

## 2. Where it starts

**Faction today.** `Faction::process(command) -> ProcessResult` — `Accepted`, `Rejected` or `Probed`. The observer is
told `(&Command, &ProcessResult)`. The configuration is the node's peer id, its genesis peers, the required count, the
suspicion threshold, the epoch and whether its membership is provisional. A node's log (§3 of the specification) is
its configuration, then one step per call, the command and its result: a line lost from the middle shows only as a
replay that departs somewhere after it, and a line lost at the end not at all.

**The siblings** (read at the owner's word, nothing of theirs changed or depended on):

- `process(command) -> Receipt { number, output }`; the observer is told `(&Command, &Receipt)`.
- `InstanceId { node, incarnation }`, given in the instance's origin; the caller guarantees an incarnation never
  repeats for its node, and a node that restarts is a new instance, created fresh under a new one. Beggar names its
  sessions after it, so that a restarted node's names never collide with its last life's; cartel tells one instance's
  receipts from another's by it.
- A vector's origin states the instance id; each step is a command and its receipt, number and output; the final
  state is what the closing query answers.
- A log line is `(number, command, output)`: a gap in the numbers proves a line lost.

**What the growth rule asks.** Faction has released 115 vectors in force, none with a number or an instance; the
rule keeps every released vector's bytes. Writing them as the siblings write theirs changes every one of them.

## 3. Decisions

**D1. A receipt, its answer the output.** `Faction::process(command) -> Receipt { number, output }`, the output the
`ProcessResult` of today — the type keeps its name; the receipt's member is the siblings' word. The observer is told
`(&Command, &Receipt)`.

**D2. Every call numbered.** The number is a count the instance keeps itself: one more than its previous receipt's,
the first 1, a `Rejected` command and a `Probe` like the rest. It needs no clock, and is deterministic: the same
configuration and the same commands give the same numbers.

**D3. The instance id, the siblings'.** `InstanceId { node, incarnation }`: the node's id — the peer id it is among
its peers — and an incarnation the caller gives it, guaranteeing it never repeats for its node. Every instance the
caller creates takes a new one and numbers from 1: a fresh node, a newcomer, a removed node made a newcomer again, a
node restarted from its genesis or from a membership. `Config::new` takes the `InstanceId` in place of the peer id,
and `Config::peer_id()` stays, the instance's node; `Newcomer::new` takes the instance id of the node it will become.
The configuration's encoding states `"instance": {"node", "incarnation"}` in place of `"peer_id"`.

**D4. The vectors, written as the siblings' — re-encoded once.** Each step is a command and the receipt it must be
answered with, `{"command", "receipt": {"number", "output"}}`; the configuration states the instance; the final
answer is the receipt the closing `Probe` must be answered with, one more than the last step's. 0.8.0 re-encodes every
vector in force so, once, changing no answer: every output is the result the vector stated, every number its step's
position, every instance the vector's peer id at incarnation 1. The growth rule gains this as a way of its own — a
**re-encoding**, of every released vector at once, never of an answer: the bytes each vector was released with are
kept under `spec/released/0.7.0/`, `released.json` records the re-encoded bytes' digest and the version that
re-encoded them, and the conformance tier checks that every re-encoded vector answers as its released bytes did.
*Rejected:* the old form kept for the old vectors, the number their steps' position — two forms of vector to read,
for ever.

**D5. A log's lines numbered.** A node's log is its configuration — its instance among it — then one line per call,
as a vector writes a step: the command and the receipt, number and output. A gap in the numbers proves a line lost,
and a replay checks them as it goes.

**D6. The view shows no number.** The receipt carries it; a `Probe`'s own receipt tells the next number to whoever
asks.

**D7. No restore.** The siblings restore an instance from a checkpoint, which carries its next number; faction has
no checkpoint — a node is created from its configuration, and one created again is a new instance (D3).

## 4. The specification, 0.8.0

- §1: the growth rule's re-encoding (D4); 0.8.0 noted as the version that re-encodes.
- §2: the instance id in the configuration, and the caller's guarantee.
- §3: one entry point, one **receipt**, its output and its number; the observer told the command and its receipt; a
  node's log numbered, a gap proving a line lost.
- §17: a vector's steps and its final answer are receipts; released vectors and their re-encoding.
- The schema: the configuration's `instance`; a step's and the final answer's `receipt`; `released.json`'s
  re-encoding mark. The matrix: unchanged — no row changes, no outcome is new.
- Vectors: all 115 in force re-encoded; the withdrawn vector 113 stays as it was withdrawn.

## 5. The crate, 0.12.0, and the protocol

- `faction::receipt::Receipt { number: u64, output: ProcessResult }`;
  `faction::instance_id::InstanceId { node: PeerId, incarnation: u64 }`.
- `Faction::process` returns a `Receipt`; `Observer::observe(&Command, &Receipt)`; `Config::new(InstanceId, …)`,
  `Config::instance()`; `Config::peer_id()` stays.
- faction-protocol: `Protocol` reads the receipt's output where it read the result; `Newcomer::new` and
  `Offer::config` take the instance id. Its messages are unchanged: a number never crosses the wire.
- Breaking, as 0.11.0 was: every caller that matches on `process`'s answer, implements `Observer`, or builds a
  `Config` changes. Raft's `raft-faction`, IBFT's integration and their Embassy builds move when the owner says.

## 6. Validation

- **The ladder:** conformance reads and checks receipts, numbers and instances, against the model and the crate,
  and checks every re-encoded vector against its released bytes; the replay tier writes and reads numbered logs, and
  a log with a line taken from its middle departs at the gap, by its number; the property tier checks the numbers
  over random commands.
- **The independent model** numbers its answers and reads the instance as the specification says, depending on
  nothing of the crate.
- **The system tests:** each node's log lines carry the number, and every node is created with an incarnation of its
  own — a restart, a rejoin or a newcomer made again takes the next.
- **The chaos tier:** unchanged in what it checks; its nodes take incarnations as the system tests give them.

## 7. Out of scope

- Restoring an instance's count from a checkpoint (D7).
- The system tests' logs replayed in the specification's form — the other open point; numbering prepares it.
- Checking an incarnation's uniqueness: it is the caller's, as the siblings' is.

## 8. Order of work

1. This proposal, accepted.
2. The specification 0.8.0 and its re-encoded vectors, the independent model, the crate and the protocol, the
   ladder's conformance and replay.
3. The harnesses: the system tests' incarnations and numbered log lines, core- and protocol-validation, the chaos
   tier.
4. The docs: ADR, ARCHITECTURE, README, ROADMAP, CHANGELOG; OPEN_POINTS loses the entry.
5. Release 0.12.0, at the owner's word.

Each step on the branch `numbered-receipts`, both gates green on every commit.

## 9. What it costs

The receipt touches every call site: about 350 calls of `process` and 60 configurations, nearly all in tests, and 9
observers; the re-encoding touches all 115 vectors in force, by a script whose result the conformance tier checks
against their released bytes. No answer changes.
