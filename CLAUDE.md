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

## Quality Gates

### Mandatory after every change to `src/` or `tests/` of any crate in the workspace

Run gates:

`just stage1`
`just stage2`

If either gate is not green, the work is not complete. Both run identically on
Windows, Linux and macOS.

Stage 1 is formatting, clippy over every target, the `no_std` check of the four
`no_std` crates against `thumbv7em-none-eabihf`, the tests, and then the system
tests: `cargo xtask system-tests` lists every test the `all_tests` binary holds
(the cases `rstest` expands among them) and runs each in a process of its own,
in parallel, through `slotgate`, every run's logs under `logs/`.

Stage 2 is `cargo xtask stage2` -- a crate under `xtask/`, gated like any other
code. It runs, in this order:

| gate | asks |
|---|---|
| `cargo stern4rust` | do the house coding rules hold, in every package (`stern4rust.toml`) |
| `cargo dry4rust` | is any code duplicated in the library crates' source (`dry4rust.toml`, floor 25 nodes, no baseline) |
| `cargo crap4rust` | is any function of `faction` or `faction-protocol` complex and untested (threshold 15) |
| `cargo llvm-cov` | is every line, region and function of the library crates covered, the ladder's tiers counted |
| `cargo twin4rust` | does every source file of the library crates have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk too high (threshold 5) |
| `cargo deny` | do the dependencies carry an advisory, a refused licence or a foreign source (`deny.toml`) |

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

Every commit message carries the grip and braintax of the two library crates,
`core/` and `protocol/`, and their ratio, scored as the commit stages them.
`.githooks/commit-msg` runs `cargo xtask grip-braintax` on the message's file,
which writes the scores in as a paragraph of their own above the trailers: each
crate's `grip_absolute_total` over its `total_braintax` and their ratio, then the
sums and theirs, under a title naming the releases of `cargo-grip4rust` and
`cargo-braintax4rust` -- scores of different releases do not compare. The files
under `core/` and `protocol/` that the index holds are exported into a scratch
directory and scored there, so nothing unstaged counts. An amend scores again, in
place of the earlier scores; a message with no words is left alone, so git still
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

### State transition test coverage — invariant

The transition matrix tests under `ladder/tests/matrix/` provide **exhaustive** coverage of every (state × command) pair. This coverage MUST be maintained — no regression, no gaps.

| State | `ParticipationObserved` | `ReadyObserved` | `LocalParticipationCompleted` | `DeadlineExpired` |
|---|---|---|---|---|
| Initial | valid (via Fresh) | valid (via Fresh) | valid | **invalid** |
| Pinging | valid | valid | valid | valid |
| Collecting | **invalid** | valid (quorum triggers, duplicate) | **invalid** | valid (`DeadlineMissed`) |
| Bootstrapped | valid (`AcknowledgeRejoin`) | **invalid** | **invalid** | **invalid** |

`Bootstrapped` is the only terminal state; a missed deadline is non-terminal (`DeadlineMissed`), reported as `PeerState::TimedOut` via a derived view flag.

- **valid** = present in `valid_transition` rstest, expects `ProcessResult::Accepted`
- **invalid** = present in per-state `*_invalid_tests.rs` rstest, expects `ProcessResult::Rejected`
- Any new state or command variant MUST add the corresponding valid/invalid cases to preserve exhaustive coverage.
- Valid transitions live in `state_transition_matrix_tests.rs`. Invalid transitions live in per-state files (`initial_invalid_tests.rs`, `pinging_invalid_tests.rs`, `collecting_invalid_tests.rs`, `bootstrapped_invalid_tests.rs`); `pinging_invalid_tests.rs` asserts Pinging rejects nothing, and `admissible_invariant_tests.rs` checks `admissible == { c ≠ Probe : accept(c) } ∪ { Probe }` per state. Common helpers (`Init`, `Assert`, `build()`, `verify()` etc.) live in `helpers.rs`.

### State transition model

Every `punch` arm MUST follow this exact pipeline in order:

1. **predicates** — pre-compute all branch-determining values (e.g. `index`, `classification`, `is_dup`)
2. **outputs** — compute the output vector via a pure function (`compute::observed_output(...)`) or equivalent
3. **new_state** — compute the new state values via a pure `match` expression that returns a tuple, with NO mutation inside the match arms (use `iter().enumerate().map().collect()` instead of `let mut v = ...; v[i] = true`)
4. **mutation** — assign the computed values back to the working variables (e.g. `phase1_confirmed = new_phase1_confirmed;`)
5. **single return** — exactly one `(outputs, Stage::<State>(Self { ... }))` expression per arm

Violations of this model (early returns, mutation inside match arms, multiple exit points) are not acceptable.
