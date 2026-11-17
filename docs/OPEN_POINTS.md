# Open Points

Status: **Phases 1 to 5 landed** — joining, addition, failure detection, removal, and
epochs and rejoin — across the full spawn/transport matrix, and the chaos validation
built on them; see `CHANGELOG.md` for what shipped and `docs/ADRs/` for the decisions.
Everything implemented, decided, or captured in an ADR has moved out of this file; only
the genuinely-open items remain below.

Companion to `ROADMAP.md`. Items move into `ROADMAP.md` (or a phase's own spec)
once decided; until then they stay here rather than clutter the roadmap with
unsettled debate. Settled architectural decisions live in `docs/ADRs/`; shipped
changes live in `CHANGELOG.md`.

---

## Future consumers (no assumptions)

The general win is a protocol-agnostic, exhaustively-covered **admission gate**:
any system where a node comes online at runtime and someone must answer "is this
peer allowed to participate, and are we ready?" can borrow the gatekeeping state
machine while keeping *policy* (who is eligible) in the protocol. Gasper
specifically: L1 activation/exit is the beacon chain's stake+queue logic, *not* a
Faction concern; the honest fit is one layer out — distributed-validator
middleware (Obol/SSV-style DV clusters) where a newly-activated validator's client
joins a local DV cluster. Other candidates to evaluate later (not commitments):
gossip/SWIM membership layers, sidecar/sharded clusters, any BFT SMR wanting a
bootstrap+join gate. Each such consumer should contribute back a published sample
integration (the ROADMAP publication checklist already asks for one).

---

## The system tests' logs, replayed

The replay tier replays logs written in-process — random command sequences and
the simulated cluster. The system tests run real nodes, in threads, tasks and
processes, over every transport, and each writes a log nothing reads back; nor
could it be replayed. It is not the log `spec/SPEC.md` §3 defines: it opens with
no configuration, its commands are Rust's debug text rather than the
specification's encoding, its results keep only the variant, the peer state and
the sizes of the two peer lists, and the node's own events share its file.

**The likely form.** Two files per node: the specification's log — the
configuration, then one step per call, valid against the schema as it stands —
and the timestamped timeline `consolidated.jsonl` merges. The encoding and the
replay leave the ladder's test support for a library of their own, which the
system tests can depend on (not the ladder's: the conformance-alone coverage gate
measures it, and the conformance tier never replays). Each system test then
replays its nodes' logs once they shut down: every line admitted by the schema,
no departure.

**What it proves.** Replayed against the crate, a log reconstructs by
construction unless the path that writes it is broken: it checks that every call
is reported once and in order, nothing lost by the writer, on every spawn and
transport. Replayed against the independent model (`crates/spec-model/`), it judges
the behaviour too — on interleavings the vectors and the simulation never
produce. The model takes the specification's encoding as it is, so a log in the
specification's form needs nothing more of it.

---

## Deferred

- `PeerId` genericization (currently `u64`) — deferred to post-Phase-6.
