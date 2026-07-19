# P0-ADR-ProtocolAgnostic

- **Status:** Accepted
- **Date:** 2026-07-19
- **Priority:** P0 (axiom)
- **Phase:** cross-cutting

## Context
`faction` is one lifecycle primitive meant to serve multiple consensus
families. Their fault models differ, and so does the quorum arithmetic each
requires — crash-fault Raft wants `N/2 + 1`, Byzantine IBFT wants `2N/3 + 1`.

## Decision
`faction` holds zero consensus knowledge. The quorum threshold — and any
value derived from a fault model — is injected by the consumer through
`QuorumPolicy` at construction, and is never computed inside the machine.

## Forcing constraints / Evidence
The fault model belongs to the consumer, not the primitive: `faction` cannot
know which threshold is correct for a given caller. Evidence the cut sits in
the right place — the identical `Some(node_count)` quorum bug was introduced
independently in *both* consumers' own code and never in `faction`; the
boundary held under real duplication pressure.

## Rejected alternatives
A convenience default computing quorum from the peer count. Rejected: it bakes
one fault model into a supposedly protocol-agnostic primitive and is wrong for
at least one consumer by construction.

## Consequences
The API takes a `QuorumPolicy`. When the member set changes, the consumer
recomputes and re-injects the threshold. `faction` only counts confirmations
against the injected value — pure bookkeeping, no judgement.

## Enforcement
No fault-model arithmetic exists anywhere in the crate; `QuorumPolicy` is the
sole source of the threshold. The formula and its tests live consumer-side.
