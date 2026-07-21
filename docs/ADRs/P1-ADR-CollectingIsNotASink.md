# P1-ADR-CollectingIsNotASink

- **Status:** Accepted
- **Date:** 2026-07-21
- **Priority:** P1 (derived)
- **Phase:** 1

## Context
`P1-ADR-TerminalStatesAreNotSinks` fixed the terminal case: a `Bootstrapped` node
answers a member's re-sent `ParticipationObserved` with `AcknowledgeRejoin`
(re-broadcast readiness) rather than swallowing it. But `Bootstrapped` is not the
only *post-local-completion* state — `Collecting` (locally done, still gathering the
readiness quorum) was left rejecting `ParticipationObserved` outright. That
rejection is a Phase-0 assumption ("my participation phase is over"), which is
*phase*-based where it should be *membership*-based: a member's ping is not asking
the node to re-open participation, it is a peer asking "are you there, and are you
ready?" A node in `Collecting` does keep re-broadcasting via `RetryReady`, so a
lagging peer eventually hears it — but only on the next timer tick, not in reply to
its ping, and under a tight deadline that lag is a miss. That is why the IBFT
integration carries a rejoin workaround that re-advertises from local-completion
onward (across `Collecting`), broader than Faction's `Bootstrapped`-only ack. Phase
1's dynamic joining makes the gap concrete: a newcomer admitted while members are
mid-`Collecting` pings them, and dropping that ping is exactly the wrong answer.

## Decision
Once a node has completed its local participation, a **member's**
`ParticipationObserved` is answered with the node's readiness (`AcknowledgeRejoin`),
never dropped — in `Collecting` exactly as in `Bootstrapped`. A non-member's
`ParticipationObserved` stays `NonMemberIgnored`. The node re-advertises; it does not
re-open participation — it stays in `Collecting` and tracks nothing new.

## Forcing constraints / Evidence
This generalizes "no silent sinks" from the terminal state to the whole
post-local-completion lifecycle: the two states that have finished local
participation (`Collecting`, `Bootstrapped`) must behave identically toward a
member's ping, or the boundary between them is an observable, surprising special
case consumers (IBFT) have to code around. Re-advertising *readiness* rather than
tracking the participation is what the peer actually needs — the bootstrap gate
downstream is the readiness quorum, not a second participation count, so `Collecting`
gains nothing by recording the peer; only the peer needs the answer. Immediacy
matters under deadlines: replying to the ping beats waiting for the next
`RetryReady`.

## Rejected alternatives
Keep rejecting (status quo) — leaves the asymmetry, the silent-sink matrix cell, and
IBFT's workaround, and gives only eventual (timer-tick) re-advertisement. Accept and
*track* the participation (add the peer to `pinging_peers`) — pure bookkeeping:
`Collecting` is past participation counting, so tracking advances nothing and muddies
the state's meaning while still needing the re-advertise to be useful.

## Consequences
`Collecting` accepts `ParticipationObserved` (its `accept` and admissible set gain
it, ordered first), and its `step` returns `AcknowledgeRejoin` for a member /
`NonMemberIgnored` for a non-member, staying in `Collecting` with unchanged counts.
IBFT's rejoin workaround can collapse to lean on this. The exhaustive matrix gains
two `Collecting × ParticipationObserved` cells (member, non-member); the previously
"invalid" rejection cell is removed.

## Enforcement
`collecting_tests::process_acknowledges_member_participation` and
`process_ignores_non_member_participation` pin the two outcomes and that the snapshot
is unchanged; the transition matrix carries the two new valid cells and the
admissible-invariant includes `ParticipationObserved` in `Collecting`'s set; the
property model replays random sequences reaching `Collecting` and asserts the same.
`system-tests`' `a_newcomer_admitted_before_bootstrap_still_converges` drives it
end-to-end. All run in the stage-1 gate.
