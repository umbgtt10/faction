# P1-ADR-NoLineTheSpecificationDoesNotAskFor

- **Status:** Accepted
- **Date:** 2026-11-12
- **Priority:** P1 (derived from `P0-ADR-SpecificationIsNormativeCrateIsReference`)
- **Phase:** cross-cutting

## Context
Coverage is usually a target approached over time, because infrastructure —
sockets, files, clocks, processes — makes some lines hard to reach from a test.
`faction` has none (`P0-ADR-PureMealyNoIO`): every line is reachable from
`process`. A line left uncovered is therefore never a hard-to-test line; it is
untested behaviour, dead code, or behaviour the specification does not ask for.
The crate came before its specification, and when the specification's vectors
were first measured alone they reached 93% of it.

## Decision
`faction` has 100% coverage — no uncovered line, region or function — under all
the test tiers together, and under the conformance vectors alone.

## Forcing constraints / Evidence
**Nothing is out of reach.** With nothing injected but an observer that answers
nothing, and one entry point, there is no line a test cannot drive.

**The vectors alone are the stronger measure.** The specification is normative:
a line its vectors never reach is behaviour it does not state — unspecified
behaviour, or dead code. Measured alone, they found three kinds: accessors
derivable from what the specification does show (whether a node is concluded,
the configuration's peer count, the result's view whatever its variant); arms
no command could reach, because admission was decided apart from the step that
matched every command; and a membership check made twice, the second never
deciding anything. Each was removed or restructured; none was excused.

## Rejected alternatives
**A percentage threshold.** Rejected: a rounded percentage hides the one line
that matters. The gate counts uncovered items, and allows none.

**Excluding code from measurement.** Rejected: an exclusion is where an
untested line would hide.

**Keeping the accessors consumers find convenient.** Rejected: each is a line
the specification does not state, and each is a one-line read of what it does —
the peer state, the peer lists, the result's variant.

## Consequences
A new line arrives with the vector that reaches it. A line no vector can reach
is either removed, or the specification grows a vector — and a row, if the line
is behaviour the matrix does not yet state. A caller that needs a convenience the
specification does not ask for keeps it on its own side, as `faction-protocol`
keeps `ResultView`.

Coverage says every line ran, not that its effect is asserted: `cargo-mutants`
answers the second.

## Enforcement
Two coverage gates in stage 2: `cargo llvm-cov` with `--fail-uncovered-lines 0`,
`--fail-uncovered-regions 0` and `--fail-uncovered-functions 0`, over every tier
(`faction`, `faction-protocol`, the ladder), and over the conformance tier alone
(`faction` and the ladder, its tests filtered to `conformance::`).
