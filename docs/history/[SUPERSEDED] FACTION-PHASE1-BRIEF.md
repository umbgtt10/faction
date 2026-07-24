# `faction` Phase 1 — Engineering Brief

**Objective:** promote `faction` from an implementation to a **normative
specification with a conformant reference implementation**, while adding dynamic
operator joining.

This document is the design contract for Phase 1. It states the decisions that
must be made deliberately, the traps that are cheap now and expensive later, and
the acceptance criteria. It is intentionally prescriptive: where a decision is
open, it says so explicitly; everywhere else, follow it.

> **Placeholders.** State, command and output names below are written as
> `StateA`, `CmdX`, etc. Substitute the real `faction` vocabulary. Do not invent
> new states or commands — the state set is defined by the existing machine plus
> the Phase 1 join transitions, and nothing else.

---

## 0. Governing principle

From v1.0, **the specification is normative and the Rust crate is a conformant
reference implementation.** Where the two disagree, the specification is correct
and the crate has a bug.

Practical consequence for Phase 1: design questions are settled in the
specification first, then implemented. If implementation reveals the spec is
wrong, the spec is amended deliberately and the change is recorded — not silently
reconciled by changing the code.

**Do not** let the crate's current Rust types define the specification's
vocabulary by default. That is the single most likely way this phase produces
documentation-with-test-vectors instead of a specification.

---

## 1. Invariants that must not be broken

These already hold and must survive the refactor. Any change that violates one is
wrong regardless of how convenient it is:

1. **Purity.** `output = F(state, input)`. No I/O, no clock reads, no randomness,
   no global state inside the machine.
2. **No wire format.** No message encoding, no transport semantics, no
   peer-to-peer behaviour. Two conformant implementations may be entirely
   wire-incompatible. This is deliberate.
3. **`no_std + alloc`**, enforced by CI. The exporter tooling and harness may be
   `std`; the core may not.
4. **Zero unsafe**, `#![deny(unsafe_code)]`.
5. **Total transition coverage.** Every `(state, command)` pair has a defined,
   tested outcome — including the pairs whose outcome is rejection.

---

## 2. The canonical model (critical decision)

### 2.1 Do not serialize Rust types directly

Do **not** put `#[derive(Serialize)]` on the core state/command/output types and
export those. Reasons:

- Rust enum variant names, struct field names and nesting are implementation
  details. Serializing them makes the vector format hostage to every internal
  refactor.
- A Go or TypeScript implementation has no reason to mirror Rust's type
  structure, and shouldn't have to.
- It quietly makes the crate normative rather than the spec — the exact inversion
  this phase exists to prevent.

### 2.2 Introduce an explicit canonical model

Create a separate, versioned, language-neutral vocabulary:

```
faction-spec-model/           # canonical names, no dependency on core internals
  states.rs                   # canonical state identifiers + canonical fields
  commands.rs
  effects.rs
  mapping.rs                  # core types  <->  canonical model
```

The canonical model is the specification's vocabulary. The mapping layer is the
only place that knows about both. If a Rust refactor changes an internal type,
only `mapping.rs` changes and the vectors are unaffected — which is the whole
point.

Naming rules for the canonical model:

- `snake_case` in the serialized form, regardless of Rust conventions
- Names describe **domain concepts**, not Rust constructs (`quorum_pending`, not
  `StatePendingQuorumEnum`)
- No abbreviations that aren't already ecosystem-standard
- Once published in v1, a name is frozen for the life of the major version

---

## 3. Output representation (the hardest design question)

Outputs are the part where "conformance suite" and "regression suite" diverge.

**Failure mode A — too concrete.** Vectors assert Rust output types. Only
`faction` can pass. The suite proves nothing about anyone else's implementation;
it is a regression suite with extra ceremony.

**Failure mode B — too abstract.** Vectors assert something like "a join was
processed." Any implementation passes. The suite proves nothing at all.

### 3.1 Required approach: a closed effect vocabulary

Define outputs as **observable effects** — a small, closed, enumerated set of
things the machine tells its caller to do. Each effect has a fixed name and a
structured parameter set drawn from the canonical model.

Illustrative shape (substitute real effects):

```json
{ "effect": "emit_join_request", "target": "n3" }
{ "effect": "admit_member",      "member": "n3" }
{ "effect": "reject_join",       "member": "n3", "reason": "quorum_would_break" }
{ "effect": "no_op" }
```

Rules:

- The effect set is **closed**. Adding an effect is a spec change, not an
  implementation detail.
- Effect parameters carry only information the caller genuinely needs. If a field
  exists because it was convenient in Rust, it does not belong in the spec.
- Rejection reasons are part of the vocabulary and must be enumerated. "The
  command was rejected" is not conformance-checkable; "rejected with reason
  `quorum_would_break`" is.

### 3.2 Ordering

Decide and document explicitly: are effects a **sequence** (order significant) or
a **set** (order irrelevant)?

Default recommendation: **sequence**, with a canonical ordering rule stated in the
spec, so comparison is a simple deep equality. A set semantics forces every
harness to implement order-insensitive comparison, which is a portability tax for
no benefit.

---

## 4. Determinism requirements

Vectors are only checkable if the machine is deterministic in every dimension a
harness can observe.

1. **Identifiers are opaque and canonical.** Use `n1`, `n2`, `n3`… in vectors, not
   ENRs, public keys or addresses. The spec defines member identity as an opaque
   comparable token. Real implementations substitute their own identity type.
2. **No wall-clock time.** If the machine has timeouts, model them as an explicit
   input command (`cmd_timeout_elapsed`) delivered by the caller, never as a
   clock read. If any current code reads time, that is a purity violation to fix
   in Phase 1.
3. **Collections are canonically ordered.** Member sets, pending change lists and
   anything else iterable must serialize in a defined order (sorted by identifier
   is fine). Otherwise vectors are non-reproducible across languages.
4. **No randomness.** Should already hold; verify.

---

## 5. Reachability (portability trap)

To test a transition out of `StateA`, a harness must first *get the
implementation into* `StateA`.

Requiring implementations to expose a "construct arbitrary state" API is a bad
demand: it forces test-only surface into everyone's public interface, and it lets
a harness construct states the machine could never actually reach.

### 5.1 Required vector shape

Every vector is **reachable-by-construction from the initial state**:

```json
{
  "id": "join.rejected.below_quorum.001",
  "spec_version": "1.0.0",
  "description": "join request rejected when admission would break quorum",
  "setup": [
    { "command": "cmd_init", "members": ["n1","n2","n3","n4"], "threshold": 3 },
    { "command": "cmd_member_offline", "member": "n4" }
  ],
  "step": { "command": "cmd_join_request", "member": "n5" },
  "expect": {
    "state": { "name": "quorum_pending", "members": ["n1","n2","n3"], "...": "..." },
    "effects": [ { "effect": "reject_join", "member": "n5", "reason": "..." } ]
  }
}
```

`setup` is a command sequence applied from the initial state, with results
unchecked. `step` is the single transition under test. `expect` is the assertion.

This means a conformant implementation only needs to expose: construct-initial,
apply-command, observe-state, observe-effects. That is a reasonable ask of any
implementation and requires no test-only surface.

### 5.2 Scenario vectors

In addition to single-step vectors, include a small number of **scenario
vectors**: longer command sequences with assertions after each step, used to
express invariants that are properties of *sequences* rather than of single
transitions (e.g. no interleaving admits two disjoint quorums).

Phase 1 scope: single-step vectors are mandatory and exhaustive; scenario vectors
cover the join flow only. Phases 3–4 expand them heavily.

---

## 6. Type-level guarantees become runtime obligations

This is the subtlest point in the phase and must be handled explicitly.

Several `faction` safety properties are enforced by Rust's type system — certain
transitions are *structurally unrepresentable*, which is stronger than a runtime
check. The specification's argument rests on this.

But **a Go implementation has no such type system.** What Rust makes impossible to
express, Go must reject at runtime. Therefore:

1. The specification must state, for each invariant, whether the reference
   implementation enforces it **statically** or **dynamically**.
2. For every statically-enforced invariant, the vectors **must** include the
   corresponding illegal transition with an expected rejection, so a dynamically
   -checked implementation can be verified.
3. The Rust reference harness may legitimately be unable to execute those
   vectors — the case is unconstructible. Mark such vectors
   `"unconstructible_in": ["reference"]` and have the Rust harness skip-with-reason
   rather than fail, while other languages must execute them.

Getting this wrong produces a suite that certifies Rust and silently exempts
everyone else from the invariants that matter most.

---

## 7. Single source of truth

The existing exhaustive matrix tests and the exported vectors **must derive from
the same data**. If the matrix is currently a set of hand-written `#[test]`
functions, that is the main refactor of this phase.

Required structure:

```
core/tests/transition_matrix/
  matrix.rs          # declarative table: the single source of truth
  tests.rs           # consumes matrix.rs, runs as cargo test
tools/vector-export/
  main.rs            # consumes matrix.rs, emits vectors/
```

The table is data. Tests and exporter are two consumers of it. Adding a
transition means adding a row, and both consumers update automatically — which is
what makes later phases free rather than doubly expensive.

---

## 8. Vector artifacts and repository layout

```
spec/
  SPECIFICATION.md          # normative document
  CHANGELOG.md              # every normative change, with rationale
vectors/
  v1/
    manifest.json           # spec_version, format_version, counts, checksums
    bootstrap/*.json
    join/*.json
    scenarios/*.json
  FORMAT.md                 # vector schema + harness contract
tools/
  vector-export/            # generator
harness/
  rust/                     # reference consumer
```

`manifest.json` carries `spec_version`, `format_version`, total vector count, and
a checksum per file — so a consumer can detect a partial or tampered vector set.

---

## 9. Harness contract

`vectors/FORMAT.md` must specify, for an implementer with no knowledge of Rust:

1. The state model: names, fields, types, canonical ordering
2. The command vocabulary: names, parameters
3. The effect vocabulary: names, parameters, ordering semantics
4. The rejection-reason enumeration
5. The four operations an implementation must expose (construct-initial,
   apply-command, observe-state, observe-effects)
6. The execution algorithm: apply `setup` unchecked, apply `step`, compare
   `expect` by deep equality
7. Comparison rules: exact equality, no tolerance, canonical ordering assumed
8. Handling of `unconstructible_in`

The test for whether this document is adequate: **could a competent Go engineer
write a conforming harness from `FORMAT.md` alone, without reading any Rust?** If
not, it is not finished.

---

## 10. Reference consumer

`harness/rust/` reads vectors from disk and drives the crate **through its public
API only**. It must not use `pub(crate)` internals, test-only constructors, or
`#[cfg(test)]` hooks.

This constraint is the point: if the reference harness needs privileged access,
then no external implementation could pass the suite, and the vectors are not
conformance vectors.

---

## 11. CI gates

Add to the existing gate set:

1. **Drift gate.** Regenerate vectors, compare byte-for-byte against committed
   output, fail on any difference. Prevents spec and implementation diverging
   silently.
2. **Completeness gate.** Assert every `(state, command)` pair in the matrix
   appears in at least one exported vector. Prevents partial export.
3. **Harness gate.** Reference harness executes the full vector set and passes,
   with skips only where `unconstructible_in` permits.
4. **Format stability gate.** Fail if a field is removed or renamed within a major
   version. Additive changes are permitted; breaking changes require a major
   bump.

Retain unchanged: 100% coverage, CRAP 0, `deny(unsafe_code)`, `no_std`.

---

## 12. Versioning policy

- `spec_version` follows semver and is independent of the crate version.
- Within a major: adding states, commands, effects or vectors is a **minor** bump.
- Renaming or removing anything in the canonical model, changing effect ordering
  semantics, or changing an existing expected outcome is a **major** bump.
- The crate declares which `spec_version` it conforms to.
- `spec/CHANGELOG.md` records every normative change with its rationale. This is
  the artifact that makes the spec credible over time; it is not optional.

---

## 13. Specification document structure

```
1. Scope and non-goals            # explicitly: no wire format, no crypto, no transport
2. Model                          # Mealy machine, purity, caller responsibilities
3. Identity and time              # opaque identifiers; timeouts as inputs
4. States                         # each state: meaning, fields, legal predecessors
5. Commands                       # each command: parameters, preconditions
6. Effects                        # closed vocabulary, ordering, rejection reasons
7. Transitions                    # the normative matrix, rendered readably
8. Invariants                     # each invariant, with static/dynamic enforcement noted
9. Structurally absent transitions # what cannot happen, and why that is the safety property
10. Conformance                   # what it means to conform; how vectors are used
11. Versioning
```

Section 9 carries the central argument and deserves the most care: for each
absent transition, state what would go wrong if it existed, and why absence is
stronger than a runtime guard.

---

## 14. Phase 1 functional scope

Dynamic operator joining:

- A node requests admission to an existing cluster at runtime
- The machine signals the request; the **caller** decides admission policy; the
  machine enforces the decision
- Admission must not break quorum invariants
- All Phase 0 vectors continue to pass unchanged

Out of scope for Phase 1 — do not implement, do not specify beyond noting as
future work: failure detection (Phase 2), commit/abort reconfiguration
(Phase 3), operator removal (Phase 4).

---

## 15. Explicit non-goals

Not delivered and not committed to in Phase 1:

- Harness implementations in languages other than Rust
- Integration with any specific DV client
- Wire format, message encoding, transport
- Key resharing or any cryptography
- Ongoing maintenance of third-party harnesses

---

## 16. Acceptance criteria

- [ ] Canonical model exists as a separate module with no dependency on core internals
- [ ] Mapping layer is the only code aware of both vocabularies
- [ ] Transition matrix is declarative data; tests and exporter both consume it
- [ ] Effect vocabulary is closed, enumerated, with enumerated rejection reasons
- [ ] Effect ordering semantics decided and documented
- [ ] All collections canonically ordered; no clock reads; no randomness
- [ ] Every vector reachable-by-construction from initial state
- [ ] Statically-enforced invariants identified and covered by rejection vectors
- [ ] `unconstructible_in` handled by the reference harness
- [ ] `FORMAT.md` sufficient for a Go engineer with no Rust knowledge
- [ ] Reference harness uses public API only
- [ ] Four CI gates green: drift, completeness, harness, format stability
- [ ] Phase 0 vectors pass unchanged
- [ ] `SPECIFICATION.md` complete, section 9 substantive
- [ ] `spec/CHANGELOG.md` initialised at v1.0.0
- [ ] Existing gates unchanged: 100% coverage, CRAP 0, no unsafe, `no_std`

---

## 17. Notes for LLM-assisted implementation

- Do not invent states, commands or effects. The vocabulary is fixed by the
  existing machine plus the Phase 1 join transitions. Ask rather than guess.
- Do not reach for `#[derive(Serialize)]` on core types. Section 2 exists
  specifically to prevent that shortcut.
- Do not add a state-injection API to make vectors easier. Section 5 exists
  specifically to prevent that shortcut.
- When the specification and the implementation disagree, stop and escalate. Do
  not silently change either to match the other.
- Prefer a boring, verbose, stable vector format over a clever compact one. It is
  a public interface with a long life.
