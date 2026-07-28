# Grip / Braintax Validation

Testability (`grip`) and cognitive-load (`braintax`) scores across every `core/`-touching
commit on `feat/phase-1-joining`, used as a real-world validation corpus for both tools.

## Methodology

**Scope: `core/` only.** 132 of the branch's 342 commits touch it, selected via
`git log -- core/`. `core-validation/`, `protocol-validation/`, and `system-tests/` are
testing/validation harnesses by their own `Cargo.toml` descriptions; `protocol/`
(`publish = false`) isn't the shippable deliverable — `core/` is the crate actually
published as `faction`.

Analyzed with [`cargo-grip4rust`](https://crates.io/crates/cargo-grip4rust) v0.7.0 and
[`cargo-braintax4rust`](https://crates.io/crates/cargo-braintax4rust) v0.11.0, both
crates.io releases. Each commit was checked out into a detached, throwaway `git worktree`
(never touching this repo's actual working tree or branch state), analyzed in place, then
the worktree was removed. Both tools parse `.rs` files directly via `syn` — no build step,
no compilation — so this covers every included commit regardless of whether it happened to
compile at that point in history.

**Timestamp** is the commit's author date (`%ad`, local time as recorded at authorship).

**Grip Total** and **Braintax Total** are `grip_absolute_total` and `total_braintax` — the
raw, unbounded sums across every function in `core/` at that commit, not the normalized
0–100 `grip_score`/`braintax_normalized`. This is a deliberate choice: `grip_absolute_total`
is a direct sum of real per-function contributions (each capped at 1.0), while
`total_braintax` is a direct sum of real per-function cognitive cost (uncapped — one bad
function can outweigh dozens of clean ones). Both totals grow with the codebase's size,
which is expected and not itself meaningful; what the **Grip/Braintax** ratio tracks is
whether testable structure is keeping pace with accumulating complexity as the codebase
grows, not whether either number is "big." This is the same `TI = grip_absolute_total /
total_braintax` pairing decided in both tools' own `docs/FORMULA.md`, in preference over
`grip_score / braintax_normalized`, since those are each already a lossy,
independently-shaped 0–100 projection rather than a direct total.

Both totals are shown in scientific notation.

## Interpretation

**The ratio holds, it doesn't erode.** Across the 132 commits, Grip/Braintax opens at
0.3242 and closes at 0.3404, fluctuating within a 0.2675–0.4015 band throughout (mean
0.343) — no sustained decline. Testable structure has kept pace with `core/`'s growth in
cognitive load over the branch's life, not fallen behind it.

**Reading the Grip Total column:** grip's `HiddenDepFinder` trusts `.clone()` on a custom
type whose fields are provably plain data, and trusts `len`/`get`/`is_empty`/`contains`/
`iter` on a custom type when the specific method called is a provably pure, zero-hidden-dep
inherent `&self` accessor — but it still cannot see everything:

- **Trait-impl methods aren't considered**, only inherent ones — a pure accessor reached
  through a local trait impl is still flagged as a hidden dependency.
- **Nested custom-method trust doesn't recurse** — a custom accessor whose own body calls
  another custom type's accessor won't have that inner call trusted.
- **Enums, generic fields, and cross-crate types** are unresolved — grip's registries only
  see structs, only see concrete type names, and only see what's inside the scanned path.

Each of these fails *safe* — a real accessor gets counted as a hidden dependency when it
isn't one, never the reverse. **Grip Total is therefore a conservative floor:** `core/`'s
true testability is at least as good as what's shown here, never worse. See grip's own
`OPEN_POINTS.md` for the current state of each gap.

## Results

| Timestamp | Commit | Grip Total | Braintax Total | Grip/Braintax | Summary |
|---|---|---|---|---|---|
| 2026-04-26 15:37 | 8ac5440 | 3.42e+01 | 1.06e+02 | 0.3242 | Convert to workspace with faction (core) and faction-validation crates |
| 2026-04-26 18:04 | f04bce7 | 3.42e+01 | 1.06e+02 | 0.3242 | Replace 16 narrative tests with single rstest state transition matrix |
| 2026-04-26 18:11 | 8d71050 | 2.89e+01 | 9.74e+01 | 0.2964 | Remove OutputBatch wrapper, use Vec<ClusterReadinessOutput> directly |
| 2026-04-26 18:37 | 70b9faa | 2.89e+01 | 9.74e+01 | 0.2964 | Rename ClusterReadiness -> Vibe |
| 2026-04-26 19:34 | 6cff733 | 4.06e+01 | 1.48e+02 | 0.2748 | Refactor Vibe into state pattern with Box<dyn VibeState> |
| 2026-04-26 19:45 | 41fa426 | 4.06e+01 | 1.48e+02 | 0.2748 | Restrict Initial::deal to only ParticipationObserved and ReadyObserved |
| 2026-04-26 19:57 | c6f5be5 | 4.06e+01 | 1.48e+02 | 0.2748 | Add state-specific tests for 100% coverage of states/ |
| 2026-04-26 20:10 | 6969a2f | 4.06e+01 | 1.27e+02 | 0.3207 | Remove dead code branches from state punch methods |
| 2026-04-26 20:31 | 5e17fd0 | 4.06e+01 | 1.27e+02 | 0.3207 | Rename Phase1 -> Pinging, Phase2 -> Collecting |
| 2026-04-27 04:51 | 4a90fce | 4.06e+01 | 1.34e+02 | 0.3038 | Refactor punch arms to if/else chains with single exit point |
| 2026-04-27 04:54 | 77f8cf9 | 4.06e+01 | 1.35e+02 | 0.3010 | Refactor Pinging LocalParticipationCompleted to single exit pattern |
| 2026-04-27 04:58 | 64ece83 | 4.54e+01 | 1.45e+02 | 0.3134 | Extract observed output computation into states/compute.rs |
| 2026-04-27 05:00 | 68ba414 | 4.54e+01 | 1.45e+02 | 0.3134 | Shorten module paths: use import instead of super:: prefixes |
| 2026-04-27 05:47 | 96e6f77 | 4.54e+01 | 1.64e+02 | 0.2775 | Pure functional state transitions: no mutation in match arms |
| 2026-04-27 15:48 | 73631db | 4.54e+01 | 1.69e+02 | 0.2689 | Refactor LocalParticipationCompleted to match state transition model |
| 2026-04-27 15:54 | 2f0f97b | 4.54e+01 | 1.67e+02 | 0.2712 | Refactor compute.rs into ObservedOutput struct with single entry point |
| 2026-04-27 16:02 | 13d07ec | 4.54e+01 | 1.70e+02 | 0.2675 | refactor: rename compute module to compute_output and move inside helpers |
| 2026-04-27 16:15 | c49063c | 4.60e+01 | 1.47e+02 | 0.3124 | refactor: extract ConfirmedSet helper to reduce complexity in Pinging/Collecting punch |
| 2026-04-27 16:34 | aeec692 | 4.60e+01 | 1.54e+02 | 0.2980 | Implement adversarial review recommendations 2-5 |
| 2026-04-27 16:42 | 68351a5 | 4.50e+01 | 1.42e+02 | 0.3175 | Replace dead punch code with unreachable!() + default deal() on trait |
| 2026-04-27 16:50 | b072ac2 | 4.40e+01 | 1.45e+02 | 0.3023 | Cache snapshot in Vibe via Cell<Option<VibeSnapshot>> |
| 2026-04-27 16:57 | 1480c53 | 4.40e+01 | 1.45e+02 | 0.3023 | Fill observer and validation harness coverage gaps for Phase 0 |
| 2026-04-27 17:10 | 554909b | 4.40e+01 | 1.45e+02 | 0.3023 | Rename Vibe -> Machine across entire workspace |
| 2026-04-27 17:31 | 061424d | 5.40e+01 | 1.74e+02 | 0.3104 | Implement state_snapshot design — states own only their active data |
| 2026-04-27 17:35 | 59a6813 | 5.40e+01 | 1.74e+02 | 0.3104 | Add 12 unit tests for state_snapshot design |
| 2026-04-27 17:56 | 1b771af | 5.40e+01 | 1.74e+02 | 0.3104 | Bump version to 0.2.0 for both workspace crates |
| 2026-04-27 18:01 | 9ac1dae | 5.40e+01 | 1.74e+02 | 0.3104 | Add CHANGELOG.md with full history to v0.2.0 |
| 2026-04-29 20:17 | 0022470 | 5.40e+01 | 1.79e+02 | 0.3012 | feat: add GetSnapshot input for querying machine state |
| 2026-04-29 21:34 | b4c5db2 | 5.40e+01 | 1.79e+02 | 0.3012 | refactor: rename Machine→Faction, MachineInput→Command, MachineOutput→Outcome, and all related types |
| 2026-04-29 21:37 | df5bdfb | 5.40e+01 | 1.79e+02 | 0.3012 | refactor: rename all machine_* files to faction_* (core + validation) |
| 2026-04-30 06:03 | 754bba6 | 5.80e+01 | 1.88e+02 | 0.3089 | Enforce AAA standard with blank-line separation across all tests |
| 2026-04-30 06:23 | 4a92263 | 5.80e+01 | 1.88e+02 | 0.3089 | Add ApplyStatus::Snapshot variant for GetSnapshot command |
| 2026-04-30 06:30 | 9a485f0 | 5.80e+01 | 1.88e+02 | 0.3089 | Add invalid_transition rstest covering Collecting, ReadyByQuorum, and ReadyByDeadline rejection paths |
| 2026-04-30 16:56 | 717bd1c | 5.80e+01 | 1.88e+02 | 0.3089 | Reorganize invalid transition tests per state with shared helpers |
| 2026-04-30 17:04 | 5b25119 | 5.80e+01 | 1.88e+02 | 0.3089 | Add initial and pinging invalid transition test files |
| 2026-04-30 17:07 | 97d7706 | 5.80e+01 | 1.88e+02 | 0.3089 | Add deadline_expired_from_collecting valid transition case |
| 2026-04-30 17:13 | 794b12f | 5.80e+01 | 1.88e+02 | 0.3089 | Rename apply -> process and ApplyStatus -> ProcessResult |
| 2026-04-30 17:24 | c9ded18 | 5.80e+01 | 1.88e+02 | 0.3089 | Rename GetSnapshot -> Probe and ProcessResult::Snapshot -> Probed |
| 2026-04-30 17:27 | d18fef8 | 5.80e+01 | 1.88e+02 | 0.3089 | Add admissible to ProcessResult::Probed |
| 2026-04-30 17:46 | df466d2 | 5.80e+01 | 1.88e+02 | 0.3089 | Make snapshot() private, replace all external calls with process(Probe) |
| 2026-04-30 17:50 | 31b46ec | 5.67e+01 | 1.84e+02 | 0.3087 | Consolidate snapshot, base_snapshot, compute_snapshot into one method |
| 2026-04-30 17:59 | 8982875 | 5.63e+01 | 1.81e+02 | 0.3116 | Remove Option wrapper from Faction::state, eliminate all unwraps |
| 2026-04-30 18:01 | 47d2f20 | 5.68e+01 | 1.79e+02 | 0.3181 | Remove Option wrapper from cached_snapshot, compute initial in constructor |
| 2026-04-30 18:03 | 11d7f17 | 5.63e+01 | 1.78e+02 | 0.3173 | Inline snapshot() method, access cached_snapshot field directly |
| 2026-04-30 18:11 | 5e4af93 | 5.63e+01 | 1.73e+02 | 0.3251 | Move confirmed_set and compute_output out of helpers/ to states/ |
| 2026-04-30 18:14 | 916f7e6 | 5.63e+01 | 1.73e+02 | 0.3251 | Add missing test files for config, quorum_policy, transition |
| 2026-04-30 19:06 | 33f072a | 5.63e+01 | 1.73e+02 | 0.3251 | Rename ReadyByDeadline -> TimedOut across codebase |
| 2026-04-30 19:08 | d13a6af | 5.63e+01 | 1.73e+02 | 0.3251 | Rename ReadyByQuorum -> Bootstrapped across codebase |
| 2026-04-30 19:13 | d1a12b4 | 5.63e+01 | 1.73e+02 | 0.3251 | Rename ReadinessExitMode variants: Quorum->Bootstrapped, Deadline->TimedOut |
| 2026-04-30 19:30 | b314449 | 5.44e+01 | 1.70e+02 | 0.3194 | Rename Snapshot -> ClusterView across entire codebase |
| 2026-04-30 19:33 | f373fb9 | 5.44e+01 | 1.70e+02 | 0.3194 | Fix State trait bound regression and add AAA to cluster_view_tests |
| 2026-04-30 19:40 | 2bec5d9 | 5.44e+01 | 1.58e+02 | 0.3447 | Merge StateClusterView trait into State, delete state_snapshot.rs |
| 2026-04-30 19:41 | af3ba4a | 5.44e+01 | 1.58e+02 | 0.3447 | Final cleanup after Snapshot->ClusterView and StateClusterView merge |
| 2026-04-30 19:45 | f5ce5b8 | 5.44e+01 | 1.58e+02 | 0.3447 | Rename ReadinessLifecycleState -> NodeState and lifecycle_state -> node_state |
| 2026-04-30 19:48 | 7628635 | 5.44e+01 | 1.58e+02 | 0.3447 | Rename Phase1Active->Pinging, Phase2Active->Collecting and related properties |
| 2026-04-30 19:52 | f954214 | 5.44e+01 | 1.58e+02 | 0.3447 | Rename quorum_threshold -> required_count across codebase |
| 2026-04-30 19:56 | 8f0ec2f | 5.44e+01 | 1.58e+02 | 0.3447 | Rename local_participation_complete -> is_pinging_completed |
| 2026-04-30 21:16 | 55b5481 | 5.42e+01 | 1.61e+02 | 0.3355 | Refactor ClusterView from peer counts to peer lists |
| 2026-04-30 21:19 | d99c229 | 5.42e+01 | 1.61e+02 | 0.3355 | Rename NodeState -> PeerState for consistency with PeerId |
| 2026-04-30 21:26 | e21962f | 5.42e+01 | 1.61e+02 | 0.3355 | Rename parameter input -> command across codebase |
| 2026-04-30 21:32 | 7a5a545 | 5.42e+01 | 1.61e+02 | 0.3355 | Add observe_query and observe_rejection to Observer trait |
| 2026-04-30 21:34 | 063b6a7 | 5.59e+01 | 1.64e+02 | 0.3405 | Remove default impls from Observer, implement in all observers |
| 2026-04-30 21:51 | 1c22210 | 5.59e+01 | 1.64e+02 | 0.3405 | Add PeerState::Fresh variant for freshly created Faction state |
| 2026-04-30 22:01 | 32ba1fd | 5.59e+01 | 1.64e+02 | 0.3405 | Rename ReadinessExitMode -> ExitMode, cleanup Outcome |
| 2026-04-30 22:06 | 0dc52d0 | 5.59e+01 | 1.64e+02 | 0.3405 | Rename readiness_exited -> is_exited and inputs -> commands in property tests |
| 2026-04-30 22:09 | 9c08f08 | 5.59e+01 | 1.64e+02 | 0.3405 | chore: some more renaming |
| 2026-04-30 22:15 | c3a9521 | 5.59e+01 | 1.64e+02 | 0.3405 | Fix remaining old names in property tests |
| 2026-04-30 22:17 | e8a2a1e | 5.91e+01 | 1.81e+02 | 0.3269 | Replace Vec<bool>+count in ConfirmedSet with dedicated Bitmap type |
| 2026-04-30 22:48 | 0e9b3d5 | 6.00e+01 | 1.82e+02 | 0.3297 | Add is_empty() to Bitmap to satisfy clippy len-without-is-empty |
| 2026-05-01 06:17 | 6081880 | 6.29e+01 | 1.84e+02 | 0.3416 | Simplify ConfirmedSet API — remove unused Pinging::new parameter, rename is_dup to is_member, drop out-of-bounds concept |
| 2026-05-01 06:23 | bdf837a | 6.29e+01 | 1.84e+02 | 0.3416 | Rename phase1/phase2 terminology to pinging/collecting throughout codebase |
| 2026-05-01 06:26 | 9e0df87 | 6.29e+01 | 1.84e+02 | 0.3416 | Rename remaining phase1/phase2 terminology to pinging/collecting |
| 2026-05-01 06:34 | b3786bb | 6.29e+01 | 1.84e+02 | 0.3416 | Remove config parameter from State::cluster_view |
| 2026-05-01 07:36 | af75b8b | 6.96e+01 | 1.93e+02 | 0.3603 | Simplify compute_output and try_confirm signatures, consolidate non-member gate, standardize state constructors |
| 2026-05-01 08:26 | bf49881 | 6.55e+01 | 1.87e+02 | 0.3506 | Inline ConfirmedSet into Vec<PeerId> in state structs |
| 2026-05-01 08:40 | 8541493 | 6.85e+01 | 1.90e+02 | 0.3605 | Introduce ObservedStep to encapsulate observation decision logic |
| 2026-05-01 08:58 | 6e40348 | 6.81e+01 | 1.90e+02 | 0.3589 | Move threshold into ObservedStep constructor, consolidate into single outputs(), add is_quorum() |
| 2026-05-01 09:17 | 261cccc | 6.81e+01 | 1.90e+02 | 0.3589 | Rename pinging_count/collecting_count to pinged_peers/collected_peers |
| 2026-05-01 09:29 | 54b6b44 | 6.81e+01 | 1.90e+02 | 0.3589 | Remove mut locals from Pinging::step and Collecting::step |
| 2026-05-01 09:36 | 45f936d | 6.91e+01 | 1.89e+02 | 0.3650 | Consolidate LocalParticipationCompleted arm into ObservedStep::new_local |
| 2026-05-01 09:46 | 0953ab5 | 6.91e+01 | 1.89e+02 | 0.3650 | Fold compute_output module into observed_step, remove unused module |
| 2026-05-01 09:48 | 5d2265a | 6.91e+01 | 1.89e+02 | 0.3650 | Restore separate compute_output module (one struct per file) |
| 2026-05-01 09:55 | ec444e0 | 6.91e+01 | 1.89e+02 | 0.3650 | Add observed_step_tests with dedicated unit coverage |
| 2026-05-01 10:02 | b068c40 | 6.15e+01 | 1.67e+02 | 0.3681 | Remove dead bitmap module, add Initial::new() test |
| 2026-05-01 14:03 | 2234198 | 6.15e+01 | 1.67e+02 | 0.3681 | Fix model bug: prev_collecting count reset removed from two handlers |
| 2026-05-01 15:38 | 89e11a5 | 6.15e+01 | 1.67e+02 | 0.3681 | fix: store peer collections in terminal states, drive system tests manually |
| 2026-05-01 18:25 | 38ac8de | 6.15e+01 | 1.67e+02 | 0.3681 | fix: correct repository URLs to faction repo |
| 2026-05-01 20:08 | 3b59bde | 6.15e+01 | 1.67e+02 | 0.3681 | fix: add Send bound to Observer and State traits, fix clippy |
| 2026-05-01 20:09 | 9e215ed | 6.15e+01 | 1.67e+02 | 0.3681 | fix(tests): replace Rc<RefCell> with Arc<Mutex> in observer_tests |
| 2026-05-01 20:30 | 8ea1cbb | 6.15e+01 | 1.67e+02 | 0.3681 | chore: add *.jsonl to gitignore, remove committed log files |
| 2026-05-02 08:36 | a97e62a | 5.92e+01 | 1.56e+02 | 0.3792 | refactor: observed_step — DRY quorum check, return &[PeerId] from confirmed_peers |
| 2026-05-02 08:42 | 922ecf7 | 6.21e+01 | 1.67e+02 | 0.3722 | fix: restore collecting.rs, apply only .to_vec() fix to confirmed_peers() calls |
| 2026-05-02 11:13 | 009150a | 6.30e+01 | 1.69e+02 | 0.3727 | refactor: pre-compute is_quorum and full outcomes in ObservedStep constructors |
| 2026-05-02 11:22 | d20b467 | 6.40e+01 | 1.67e+02 | 0.3820 | refactor: merge ObservedOutput::new + compute_outcome, pre-compute outcome |
| 2026-05-02 11:24 | 6d297a5 | 6.30e+01 | 1.66e+02 | 0.3792 | refactor: remove into_outcome, use outcome().clone() instead |
| 2026-05-02 11:41 | 9bc9ec8 | 6.26e+01 | 1.66e+02 | 0.3776 | refactor: remove redundant ReadyQuorumReached outcome |
| 2026-05-02 11:42 | 3753a95 | 6.26e+01 | 1.66e+02 | 0.3776 | fix: remove remaining ReadyQuorumReached references from observed_step and tests |
| 2026-05-02 11:47 | ad123d0 | 6.26e+01 | 1.66e+02 | 0.3776 | chore: stage before renaming Exited/ExitMode |
| 2026-05-02 11:51 | 7ceaaa3 | 6.26e+01 | 1.66e+02 | 0.3776 | refactor: rename Exited→Concluded, ExitMode→Conclusion |
| 2026-05-02 11:52 | 07ca755 | 6.26e+01 | 1.66e+02 | 0.3776 | fix: remaining rename references in test files and lib.rs |
| 2026-05-02 14:30 | b34f63b | 6.07e+01 | 1.57e+02 | 0.3867 | refactor: remove FreshnessPolicy and simplify Classification/Command/Outcome |
| 2026-05-02 14:39 | 8f1209d | 5.69e+01 | 1.51e+02 | 0.3758 | refactor: inline compute_output into observed_step, remove ObservedOutput wrapper |
| 2026-05-02 14:48 | 79f0fbe | 6.26e+01 | 1.58e+02 | 0.3968 | refactor: split ObservedStep into PingingStep, CollectingStep, LocalCompletionStep |
| 2026-05-02 14:57 | c9b4b7c | 6.45e+01 | 1.61e+02 | 0.4015 | refactor: add compute_new_state helpers and decouple from step structs |
| 2026-05-02 15:08 | a10028e | 6.37e+01 | 1.59e+02 | 0.4002 | refactor: rename CollectingStep→ReadyStep, make threshold required, remove dead code |
| 2026-05-02 16:59 | a360973 | 6.37e+01 | 1.59e+02 | 0.4002 | test: add timer integration tests for InMemoryTimer and RealTimer |
| 2026-05-02 22:18 | d317453 | 6.26e+01 | 1.61e+02 | 0.3883 | refactor: move PeerId to types.rs, make state struct fields private with constructors |
| 2026-05-03 05:20 | f08186f | 6.26e+01 | 1.61e+02 | 0.3883 | refactor: align test names to <method>_<description>_<outcome> convention |
| 2026-05-03 05:41 | b6f8ad6 | 6.26e+01 | 1.61e+02 | 0.3883 | refactor: align variable names with types — no more single-letter vars, machine→faction, coordinator→faction |
| 2026-05-03 05:58 | 89fe69e | 6.26e+01 | 1.61e+02 | 0.3883 | refactor: eliminate remaining single-letter vars — m→faction, s→cluster_view, t→translator, coordinator→faction |
| 2026-05-03 06:08 | 144811a | 6.26e+01 | 1.61e+02 | 0.3883 | fix: add missing AAA markers to process_participation_observed_duplicate |
| 2026-05-03 06:10 | b406e2b | 6.26e+01 | 1.61e+02 | 0.3883 | chore: some renaming in the tests |
| 2026-05-03 06:37 | 9b28974 | 5.97e+01 | 1.52e+02 | 0.3927 | refactor: most minor issues fixed |
| 2026-05-03 18:59 | af77212 | 5.97e+01 | 1.52e+02 | 0.3927 | Remove Send bound from Observer; adapt system-tests threading |
| 2026-05-03 19:30 | e1b32e4 | 5.97e+01 | 1.52e+02 | 0.3927 | chore: bump all crates to 0.3.2, consolidate Cargo.toml metadata |
| 2026-05-09 15:07 | 7f996df | 5.97e+01 | 1.52e+02 | 0.3927 | Add compelling module-level doc with runnable example to lib.rs |
| 2026-05-09 15:18 | 941c7e0 | 5.97e+01 | 1.52e+02 | 0.3927 | Add IBFT/Raft bootstrapping gap paragraph to README |
| 2026-07-19 13:48 | a00294a | 5.97e+01 | 1.52e+02 | 0.3927 | Add admissible to ProcessResult::Accepted for a consistent self-describing entry point |
| 2026-07-19 14:04 | 9f4e481 | 5.97e+01 | 1.52e+02 | 0.3927 | Assert the returned cluster_view equals a subsequent probe in the valid matrix |
| 2026-07-19 14:09 | 67576dd | 5.97e+01 | 1.52e+02 | 0.3927 | Split transition-matrix test helpers into builder and assertions modules |
| 2026-07-19 14:58 | 83e4ac3 | 5.97e+01 | 1.52e+02 | 0.3927 | Documentation sweep: fix drift, wire in the ADRs, correct the examples |
| 2026-07-19 15:15 | e5934e5 | 5.97e+01 | 1.52e+02 | 0.3927 | Remove the faction::PeerId re-export; import faction::types::PeerId directly |
| 2026-07-19 15:24 | 0bde404 | 5.97e+01 | 1.52e+02 | 0.3927 | Compile and run the README quick start as a doctest |
| 2026-07-19 17:06 | 89155e7 | 5.97e+01 | 1.52e+02 | 0.3927 | Fix single-node clusters: Initial accepts LocalParticipationCompleted |
| 2026-07-19 17:30 | a194b76 | 5.97e+01 | 1.56e+02 | 0.3832 | Phase 1: a bootstrapped node re-advertises readiness to still-trying peers |
| 2026-07-19 17:58 | d6c9efa | 5.72e+01 | 1.53e+02 | 0.3735 | Phase 2: DeadlineExpired is non-terminal; remove the TimedOut state |
| 2026-07-19 18:45 | 1470c17 | 5.72e+01 | 1.53e+02 | 0.3735 | Strengthen the transition-matrix rejection tests |
| 2026-07-20 05:38 | a24a370 | 5.95e+01 | 1.76e+02 | 0.3376 | feat: dynamic joining increments 1–7 implemented and green. Next step system tests |
| 2026-07-20 16:10 | c339455 | 6.41e+01 | 1.88e+02 | 0.3404 | refactor(core): split ClusterView into a builder and a pure DTO; add observable members |
| 2026-07-20 16:26 | 324110c | 6.41e+01 | 1.88e+02 | 0.3404 | chore: correct license headers from Apache-2.0 to MIT (SPDX) |
| 2026-07-21 07:01 | c3638cb | 6.41e+01 | 1.88e+02 | 0.3404 | feat(core): Collecting re-advertises readiness to a member's ping (decision #6) |
| 2026-07-21 07:15 | c68a1c2 | 6.41e+01 | 1.88e+02 | 0.3404 | feat(system-tests): scenario 7 — a sub-quorum cluster recovers via a join after timeout |
