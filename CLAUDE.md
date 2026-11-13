# Faction

## Meaning

`faction` is self-contained.

Do not assume or rely on any other sibling repository or crate.

## Boundary Rule

This repository is **SELF-CONTAINED**.

The LLM **SHALL NOT cross its boundaries without asking**.

That means:
- do not inspect, edit, or rely on files outside `faction/` unless the user explicitly asks
- do not pull assumptions from sibling repositories or crates
- do not propose cross-repository changes by default

## Layout

The workspace is laid out by role (`docs/ADRs/P2-ADR-WorkspaceLaidOutByRole.md`):

| directory | holds |
|---|---|
| `spec/` | the normative specification: data and prose, read by the ladder and the model, never built |
| `crates/` | the library -- `core/` (`faction`, published), `protocol/` (`faction-protocol`) and `spec-model/` (`faction-spec-model`) -- every gate at full strength, scored on every commit |
| `validation/` | everything that checks the library -- `ladder/`, `core-validation/`, `protocol-validation/` and `system-tests/` |
| `xtask/` | the gates |

A crate the library gains goes under `crates/`, and joins the gates' and the
scorer's lists of the library; a package that checks the library goes under
`validation/`. A directory is named for its package without the `faction-`
prefix, `core/` holding `faction` itself.

## Quality Gates

### Mandatory after every change to `src/` or `tests/` of any crate in the workspace

Run gates:

`just stage1`
`just stage2`

If either gate is not green, the work is not complete. Both run identically on
Windows, Linux and macOS.

Stage 1 is formatting, clippy over every target, the `no_std` check of the five
`no_std` crates against `thumbv7em-none-eabihf`, the tests, and then the system
tests: `cargo xtask system-tests` lists every test the `all_tests` binary holds
(the cases `rstest` expands among them) and runs each in a process of its own,
in parallel, through `slotgate`, every run's logs under `logs/`.

Stage 2 is `cargo xtask stage2` -- a crate under `xtask/`, gated like any other
code. It runs, in this order:

| gate | asks |
|---|---|
| `cargo stern4rust` | do the house coding rules hold, in every package (`stern4rust.toml`) |
| `cargo dry4rust` | is any code duplicated in the library's source (`dry4rust.toml`, floor 25 nodes, no baseline) |
| `cargo crap4rust` | is any function of the library -- `faction`, `faction-protocol`, `faction-spec-model` -- complex and untested (threshold 15) |
| `cargo llvm-cov` | is every line, region and function of the library covered, the ladder's tiers counted |
| `cargo llvm-cov`, conformance alone | is every line, region and function of `faction` and of the specification's model reached by the specification's vectors alone (`P1-ADR-NoLineTheSpecificationDoesNotAskFor`) |
| `cargo twin4rust` | does every source file of the library have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk too high (threshold 5) |
| `cargo deny` | do the dependencies carry an advisory, a refused licence or a foreign source, and does anything but the crates written for it take `faction` (`deny.toml`) |

`cargo install just`
`cargo install slotgate`
`cargo install cargo-llvm-cov`
`cargo install cargo-stern4rust`
`cargo install cargo-dry4rust`
`cargo install cargo-crap4rust`
`cargo install cargo-twin4rust`
`cargo install cargo-iceberg4rust`
`cargo install --locked cargo-deny`

### Every commit carries the library's scores

Every commit message carries the grip and braintax of the library's crates,
`core`, `protocol` and `spec-model` under `crates/`, and their ratio, scored as the
commit stages them. `.githooks/commit-msg` runs `cargo xtask grip-braintax` on the
message's file, which writes the scores in as a paragraph of their own above the
trailers: each crate's `grip_absolute_total` over its `total_braintax` and their
ratio, then the sums and theirs, under a title naming the releases of
`cargo-grip4rust` and `cargo-braintax4rust` -- scores of different releases do
not compare. The files under them that the index holds are exported into a
scratch directory and scored there, so nothing unstaged counts. An amend scores
again, in place of the earlier scores; a message with no words is left alone, so git still
aborts its commit; a scoring that fails stops the commit, and `--no-verify` skips
it, deliberately -- never use it. Git runs a repository's own hooks only where a
clone asks it to: `just hooks`, once a clone, on each machine.
`cargo xtask grip-braintax` with no file prints the scores.

`cargo install cargo-grip4rust`
`cargo install cargo-braintax4rust`

### Orthogonality, trait surface and cognitive complexity

**When changing productive code, always maximize orthogonality and testable surface through traits, and minimize cognitive complexity.**

Specifically:
- prefer extracting behavior behind traits so individual pieces can be tested and swapped independently
- prefer small, focused methods with a single responsibility over large methods with many branches
- prefer named structs with methods over free functions operating on external state
- when `crap4rust` or a reviewer flags a function as too complex, reduce it by extracting internal structs with methods and adding integration coverage — not by extracting standalone helper functions
- never increase cognitive complexity to pass a test; find the root cause and fix it there
- when introducing a new protocol dependency seam, place the contract in `traits/`, place the protocol-facing state/data model parallel to the protocol, and place the concrete implementation in its own dedicated implementation area
- make constructors depend on traits, not directly on concrete implementations
- ALL dependencies are injected through the SINGLE constructor and stored in the struct
- apply the same split recursively to nested dependencies: trait first, state/data model second, concrete implementation third

### User coding standards

- **one struct per file** — each struct gets its own file
- **do not use fully qualified paths** — use `use` imports instead
- **no unit tests** — only integration tests are allowed
- **no unnecessary comments** — code should be self-documenting
- **consolidate functions** — place scattered functions inside structs
- **no `&mut` parameters** — prefer return values over mutation
- **`pub mod` only in `mod.rs` and `lib.rs`** — no nested `pub mod`
- **test file naming** — one test file per source file, named `<source file name>_tests.rs`
- **`all_tests.rs`** — reference test files one by one, no `#[path = ...]`
- **AAA structure** — arrange tests with `Arrange`, `Act`, `Assert` sections separated by blank lines
- **`// Arrange & Act`** — use if there is no separate `Arrange`
- **`// Act & Assert`** — use if there is no separate `Act`
- **copyright header** — add the repository copyright and license header to every Rust source file
- **test naming**: `<method>_<description>_<outcome>` where `<method>` is the function called in the Act section. Property tests are exempt.
- **variable naming**: match the type. `<StructName>` → `<name>`, `Config` → `config`. No single-letter vars. No legacy or cryptic names.

### The specification comes first

The specification in `spec/` is normative and the crate is one implementation of it
(`docs/ADRs/P0-ADR-SpecificationIsNormativeCrateIsReference.md`). A change of behaviour
changes the specification first — the prose, the schema, the decision table's rows, a
vector for every row it adds or changes, and its version — then the crate
(`docs/ADRs/P2-ADR-SpecifiedBeforeAnyNewPhase.md`). The specification sits at the
repository root, read by the ladder and never by the crate. Vectors are written by hand
and checked against a model independent of the crate before the crate runs them; never
export them from the crate.

The model is `crates/spec-model/` (`faction-spec-model`,
`docs/ADRs/P1-ADR-IndependentModelOfTheSpecification.md`): `no_std`, free of unsafe
code, and held to every gate the library is held to. It is written from `spec/SPEC.md`
and `spec/matrix.json`, never from `crates/core/`, and depends on nothing of `faction` --
`deny.toml` refuses it -- so the two meet only in the specification's encoding, in the
ladder. It reads the decision table and answers a command by the first row that
applies; what it owns is what the table cannot say, in `rules/rule_book.rs`: each row's
situation and change. A row the matrix gains needs its rule there, or the model's
conformance tier fails; a command, an outcome or a stage the vocabulary gains is named in
`vocabulary/`.

### State transition test coverage — invariant

`spec/matrix.json` gives every (stage × command) pair its row: what each stage admits,
and the rows in check order. The conformance tier (`validation/ladder/tests/conformance/`) runs
every vector, and every row is exercised by one. The transition matrix tests under
`validation/ladder/tests/matrix/` cover every (stage × command) pair, each case naming the row it
exercises and checked against it. This coverage MUST be maintained — no regression, no
gaps.

- Valid transitions live in `state_transition_matrix_tests.rs` and expect the row's
  result; rejected commands live in per-state files (`initial_invalid_tests.rs`,
  `collecting_invalid_tests.rs`, `bootstrapped_invalid_tests.rs`) under the row
  `not-admitted`; `pinging_invalid_tests.rs` asserts Pinging rejects nothing, and
  `admissible_invariant_tests.rs` checks each stage's admissible set against the table's
  `admits` and against `{ c ≠ Probe : c is not rejected } ∪ { Probe }`.
- Any new state or command variant MUST add its rows to the table first, then the
  matching cases here.
- `Bootstrapped` is the only terminal state; a missed deadline is non-terminal
  (`DeadlineMissed`), reported as `PeerState::TimedOut` via a derived view flag.

### State transition model

Every `punch` arm MUST follow this exact pipeline in order:

1. **predicates** — pre-compute all branch-determining values (e.g. `index`, `classification`, `is_dup`)
2. **outputs** — compute the output vector via a pure function (`compute::observed_output(...)`) or equivalent
3. **new_state** — compute the new state values via a pure `match` expression that returns a tuple, with NO mutation inside the match arms (use `iter().enumerate().map().collect()` instead of `let mut v = ...; v[i] = true`)
4. **mutation** — assign the computed values back to the working variables (e.g. `phase1_confirmed = new_phase1_confirmed;`)
5. **single return** — exactly one `Some((outputs, Stage::<State>(Self { ... })))` expression per arm; the commands a stage does not admit, a probe among them, end in one arm answering `None`

Violations of this model (early returns, mutation inside match arms, multiple exit points) are not acceptable.
