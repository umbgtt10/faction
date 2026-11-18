# P1-ADR-APeerIdOfTheCallersOwnType

- **Status:** Accepted
- **Date:** 2026-11-18
- **Priority:** P1 (derived from `P0-ADR-ProtocolAgnostic`)
- **Phase:** cross-cutting, after Phase 6

## Context
A peer id was `u64`, `faction::types::PeerId`. A caller that knows its peers by something else -- a UUID, a key's
digest, a name -- had to keep a table beside the machine and translate every command and every answer. The machine
never needed a number: it copies a peer id, and asks whether two are equal, and nothing more. It never orders one,
hashes it or reads its value; the lists a view carries are in arrival order
(`P2-ADR-EffectSequenceOrderingAndCanonicalCollectionOrder`). The limitation stood as L7 in `docs/ARCHITECTURE.md`.

## Decision
A peer id is any type implementing `PeerId` (`faction::instance::peer_id`), a marker trait:
`Copy + Eq + Debug + Send`, no items. The unsigned integers implement it; a type of the caller's own does once it
says so, `impl PeerId for Name {}`, and a foreign one -- a UUID, `[u8; 16]` -- does in a newtype, by Rust's orphan
rule. There is no blanket impl. Every public type that holds a peer id takes it as a parameter, `u64` its default:
`Faction<P: PeerId = u64>`, `Config`, `Command`, `Outcome`, `ClusterView` and its builder, `Members`, `InstanceId`,
`Reconfiguration`, `Receipt`, `ProcessResult`, `Stage` and its states, the traits `State` and `Observer`; and
faction-protocol's `Protocol`, `Newcomer`, `Offer`, `Commit`, its messages, and the traits `Transport` and `Timer`.
A type that holds none -- `EpochRule`, `MessageTranslator` -- takes it on the methods that do. Epochs and
incarnations stay `u64`: they are counters, not identities. The specification says a node compares peer ids for
equality only (§1); its vectors write them as numbers, and its model keeps `u64`.

## Forcing constraints / Evidence
The ladder's property tier answers random command sequences twice, on `u64` ids and on the same ids renamed one to
one into eight masked bytes that print as the number they rename, and every answer prints alike: the machine reads
nothing of a peer id but its equality. The core's tests conclude a node on a type of a caller's own and on every
unsigned integer. With `u64` the default, a caller writing `Faction` or `Config` means what it meant; one building
a configuration from integer literals alone names the type once, `let config: Config = ...`, as the literals would
otherwise default to `i32`, which is no peer id.

## Rejected alternatives
**A blanket impl over every type with the bounds.** Rejected at the owner's word: the house rules allow none. A
type is a peer id because it says so.

**The bounds spelled on every item, no trait.** Rejected: some sixty items would repeat them, and a change to them
would touch every one.

**`Clone` rather than `Copy`.** Rejected: a heap id would cost a clone, and in a `no_std` machine possibly an
allocation, wherever an id moves; a caller holding string names maps them to small handles.

**`Default` in the bounds, to keep the admissible commands' placeholder peer.** Rejected: an identifier has no
default. The admissible commands are listed by kind instead (`P1-ADR-SingleEntryPoint`, amended).

## Consequences
**Breaking:** `faction::types::PeerId` is gone; a caller imports the trait, and every public type gains a parameter
it may leave at its default. The system tests, the chaos tier and the validation packages keep `u64`.

## Enforcement
`validation/ladder/tests/property/peer_id_tests.rs` (renamed ids answer alike);
`crates/core/tests/faction_tests.rs` (a caller's own type, every unsigned integer).
