# P1-ADR-ConfigIsImmutableGenesis

- **Status:** Accepted
- **Date:** 2026-07-20
- **Priority:** P1 (derived)
- **Phase:** 1

## Context
Through Phase 0 the peer set was fixed at construction: every membership question
read `Config.peers`, and `Config` was immutable, so "who is a member" and "what
the cluster was seeded with" were the same fact. Phase 1 (dynamic joining) breaks
that identity — a peer admitted at runtime must have its later signals counted —
which forces a decision about *where the now-mutable member set lives* and *how it
is allowed to change*. Two symmetric traps: conflating "genesis" (what we started
with) with "forever-fixed" (the exact limitation Phase 1 removes), and making
`Config` itself mutable to accommodate growth, which would smuggle uncoordinated
shared-state hazards in through a side door.

## Decision
`Config` is the immutable **genesis** seed; the live member set is authoritative
state carried in each `State` — a `Members` value object seeded from
`Config.peers` — and it changes only through an admission `Command`, never through
a setter and never by mutating `Config`.

## Forcing constraints / Evidence
Determinism (`P0-ADR-DeterministicReplayable`): `output = F(state, input)`. A
membership setter is ambient mutation outside the command stream — it breaks
replay. Growth must therefore be a `Command` (`JoinApproved`), gated by the same
`accept`/`step` machinery and `Observer` routing as every other input. Purity
(`P0-ADR-PureMealyNoIO`) with state-as-trait-object (`P2-ADR-StateAsTraitObject`):
`step` returns the next state, so a state that already carries the grown `Members`
keeps `step` a pure state→state function; a machine-owned set could not be mutated
from inside `step` and would have to be returned separately or inferred from the
`MemberAdmitted` outcome. The roadmap is membership-dominated (Phases 2–6 are all
membership work), so the authoritative mutable set belongs where the transitions
happen.

## Rejected alternatives
Machine-owned membership (representation B, a `Members` field on the `Faction`
wrapper) — forces `process` to infer growth from outcomes and splits the state
vector across the boxed state and the wrapper, breaking self-containment. A mutable
`Config` / a `Config::admit` setter — bypasses the machine's input gating (no
`accept`/`step`, no matrix coverage, no `Observer` routing) and collapses "genesis"
and "live" into one mutable cell, exactly the uncoordinated shared-state hazard
Phases 3–4 exist to prevent. Recomputing membership into `cluster_view` — that is a
derived read-model rebuilt each step; authoritative mutable state cannot live in a
projection.

## Consequences
`Initial` stops being a unit struct: it carries the genesis `Members` and threads
it into `Pinging`. Every state gains a `Members` field and its constructor a
parameter — a mechanical ripple into the tests, the same shape of change
`Collecting::new` already absorbed in Phase 0. The live `is_member` checks
(`Pinging`, `Collecting`, `Bootstrapped`, and the shared `JoinStep`) read the
carried set; `Config.peers()` is used only to seed the initial `Members` and to
replay/reset, and `Config` stays immutable. **Scope carve-out:** this ADR fixes
*membership*, not the quorum *threshold*. Phase 1 admits members while the
threshold stays fixed; whether the threshold may later change is the separate,
still-open quorum question (a coordinated Phase 3 concern) and remains deferred.
That carve-out is what unblocks writing this ADR.

## Enforcement
`join_tests::participation_from_an_admitted_peer_is_accepted` proves a signal that
was `NonMemberIgnored` before admission becomes `ParticipationAccepted` after — so
growth is real and carried in state; `admission_does_not_change_the_quorum_threshold`
pins the scope carve-out. The exhaustive `(state, command)` matrix covers all three
join edges in every state, the admissible-invariant keeps them in every state's
admissible set, and a property-model replays random join-interleaved sequences. All
run in the stage-1 gate.
