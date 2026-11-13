# P2-ADR-WorkspaceLaidOutByRole

- **Status:** Accepted
- **Date:** 2026-11-13
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context
The workspace holds the specification, the library — the crate, its protocol
runtime and the specification's independent model — four packages that check
the library, and the crate that runs the gates. Each role is held to different
gates: the library to every gate at full strength and scored on every commit,
the packages that check it to the house rules and the lints. The tree is where a
reader meets the roles first.

## Decision
The workspace is laid out by role: `spec/` holds the specification, `crates/`
the library, `validation/` every package that checks the library, and `xtask/`
the gates.

| directory | packages |
|---|---|
| `crates/core/` | `faction`, published |
| `crates/protocol/` | `faction-protocol` |
| `crates/spec-model/` | `faction-spec-model` |
| `validation/ladder/` | `faction-ladder` |
| `validation/core-validation/` | `faction-core-validation` |
| `validation/protocol-validation/` | `faction-protocol-validation` |
| `validation/system-tests/` | `faction-system-tests` |
| `xtask/` | `xtask` |

## Forcing constraints / Evidence
The gates name the library as these three crates already: CRAP, the mirrored
tests, the file risk and the commit scores measure exactly them, and the
duplication gate scans exactly their source. The layout states what the gates
enforce. `spec/` is normative data, versioned apart from the crate, read and
never built; a package inside it would make it something else.

## Rejected alternatives
**Every package at the root**, beside the specification, the documents and the
gates' configuration: the roles were stated only in the README's table.

**The model under `spec/`**: it is the specification's, but `spec/` is data
read by the ladder and the model, and stays so.

**Directories named for their packages** (`crates/faction/`,
`validation/faction-ladder/`): every path renamed for the one directory,
`core/`, whose name differs from its package's.

## Consequences
A crate the library gains goes under `crates/` and joins the library's lists in
`xtask/` — the gates name packages, and the layout selects nothing on its own.
A package that checks the library goes under `validation/`. Packages reach the
specification and the README two directories up.

## Enforcement
The workspace manifest's members. `xtask`'s `LibraryScorer` scores the crates
under `crates/`, its tests pinning the paths it lists, exports and scores.
`dry4rust.toml` scans everything but `validation/` and `xtask/`.
