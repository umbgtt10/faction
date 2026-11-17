# P1-ADR-TheLogIsTheRecord

- **Status:** Accepted
- **Date:** 2026-11-17
- **Priority:** P1 (derived from `P0-ADR-DeterministicReplayable` and `P2-ADR-TotalObservability`)
- **Phase:** between 5 and 6, at the owner's word

## Context
A node's reports, in order, are its log (`P2-ADR-TotalObservability`), and a log replays into a node created from its
configuration (`P0-ADR-DeterministicReplayable`). But a result carried no number: a line lost from the middle showed
only as a replay that departed somewhere after it, and a line lost at the end — or a probe, which changes nothing —
not at all. Nor did anything tell one run's log from another's: a node restarted wrote the same peer id. Faction's
siblings, beggar and cartel, number every receipt and name every instance; the 2026-11-16 consistency check found
this the one difference of shape between the three that is not faction's state machine.

## Decision
Every call is answered with a **receipt**: its output — the result, `Accepted`, `Rejected` or `Probed` — and a number
one more than the instance's last, the first being 1, a rejected command and a probe like the rest. Every node is an
**instance**: its node, its peer id, and an incarnation the caller gives it, guaranteeing it never repeats for that
node, so a node created again — restarted, or a removed node invited back — is a new instance, numbering from 1. A
node's log is its configuration, its instance among it, then each call's command and receipt: a gap in the numbers
proves a line lost. The shape is the siblings': `Receipt { number, output }`, `InstanceId { node, incarnation }`, the
observer told the command and the receipt.

## Forcing constraints / Evidence
**A lossy log looks complete without numbers.** With a number on every receipt, the replay tier now finds a probe cut
from a log's middle, by the gap after it; before, nothing showed.

**The node can count its calls itself.** A number is a count, not a time: it needs no clock, and the same
configuration and the same commands give the same numbers, as every answer is
(`P0-ADR-DeterministicReplayable`).

**An incarnation is the caller's.** Only the caller knows that it created the node again; the machine, holding no
storage (`P1-ADR-StatefulPersistencyFree`), could not tell. As with the siblings, its uniqueness is never checked.

## Rejected alternatives
**Vectors not writing the number**, it being the step's position: every released vector kept byte for byte, but
faction's vectors written otherwise than the siblings', and a receipt the vectors never stated. The owner chose the
siblings' form, and the specification's growth rule gained a re-encoding — every released vector at once, never an
answer.

**Naming the receipt's answer `result`**, faction's word: the owner chose the siblings' `output`; the type keeps its
name, `ProcessResult`.

## Consequences
Specification 0.8.0, for the crate's 0.12.0, a breaking release: `Faction::process` returns a `Receipt`,
`Observer::observe` takes it, and `Config::new` an `InstanceId`; faction-protocol's `Newcomer::new` and
`Offer::config` take the instance id. Every vector in force is re-encoded once, its released bytes kept under
`spec/released/0.7.0/`. The system tests' node logs carry each call's incarnation and number. Faction restores no
count from a checkpoint: it has none.

## Enforcement
The conformance tier checks every step's receipt and the closing one, numbers included, against the independent model
and the crate, every re-encoded vector against its released bytes, and every configuration read into the crate and
back; the replay tier finds a line cut from a log by its number.
