# Faction — Copilot Instructions

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

`powershell -File scripts\run_stage_1.ps1`
`powershell -File scripts\run_stage_2.ps1`

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

- one struct per file
- unit tests are not allowed. Only integration tests are
- no unnecessary comments in code
- consolidate scattered functions inside structs as appropriate
- no `&mut` input parameters; prefer return values
- only use `pub mod` in `mod.rs` and `lib.rs`
- split test files so there is one test file per source file, named `<source file name>_tests.rs`
- in `all_tests.rs`, reference test files one by one without `#[path = ...]`
- apply AAA (`Arrange`, `Act`, `Assert`) structure to tests with blank-line separation between the three sections
- use `// Arrange & Act` if there is no separate `Arrange`
- use `// Act & Assert` if there is no separate `Act`
- add the repository copyright and license header to every Rust source file

### State transition model

Every `punch` arm MUST follow this exact pipeline in order:

1. **predicates** — pre-compute all branch-determining values (e.g. `index`, `classification`, `is_dup`)
2. **outputs** — compute the output vector via a pure function (`compute::observed_output(...)`) or equivalent
3. **new_state** — compute the new state values via a pure `match` expression that returns a tuple, with NO mutation inside the match arms (use `iter().enumerate().map().collect()` instead of `let mut v = ...; v[i] = true`)
4. **mutation** — assign the computed values back to the working variables (e.g. `phase1_confirmed = new_phase1_confirmed;`)
5. **single return** — exactly one `(outputs, Box::new(Self { ... }))` expression per arm

Violations of this model (early returns, mutation inside match arms, multiple exit points) are not acceptable.
