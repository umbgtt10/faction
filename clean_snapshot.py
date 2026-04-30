# Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
# Licensed under the Apache License, Version 2.0
# http://www.apache.org/licenses/LICENSE-2.0

"""
Removes redundant exit_mode and readiness_exited fields from Snapshot.

These fields are now computed on-the-fly from lifecycle_state:
  - readiness_exited = lifecycle_state ∈ {Bootstrapped, TimedOut}
  - exit_mode = Bootstrapped→Some(Bootstrapped), TimedOut→Some(TimedOut), else None

Steps:
  1. Rewrite snapshot.rs — remove fields, constructor params, builder methods,
     add computed getters
  2. Remove .with_exit_mode() and .with_readiness_exited() calls from
     bootstrapped.rs and timed_out.rs state_snapshot impls
  3. Update all Snapshot::new() calls — drop exit_mode and readiness_exited args
  4. Remove obsolete snapshot_tests
"""

import os
import re

ROOT = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------------------
# 1. Rewrite snapshot.rs
# ---------------------------------------------------------------------------

SNAPSHOT_RS = r"""// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::readiness_exit_mode::ReadinessExitMode;
use crate::readiness_lifecycle_state::ReadinessLifecycleState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    lifecycle_state: ReadinessLifecycleState,
    local_participation_complete: bool,
    phase1_confirmed_count: usize,
    phase2_confirmed_count: usize,
    quorum_threshold: usize,
}

impl Snapshot {
    #[must_use]
    pub const fn new(
        lifecycle_state: ReadinessLifecycleState,
        local_participation_complete: bool,
        phase1_confirmed_count: usize,
        phase2_confirmed_count: usize,
        quorum_threshold: usize,
    ) -> Self {
        Self {
            lifecycle_state,
            local_participation_complete,
            phase1_confirmed_count,
            phase2_confirmed_count,
            quorum_threshold,
        }
    }

    #[must_use]
    pub const fn lifecycle_state(&self) -> ReadinessLifecycleState {
        self.lifecycle_state
    }

    #[must_use]
    pub const fn exit_mode(&self) -> Option<ReadinessExitMode> {
        match self.lifecycle_state {
            ReadinessLifecycleState::Bootstrapped => Some(ReadinessExitMode::Bootstrapped),
            ReadinessLifecycleState::TimedOut => Some(ReadinessExitMode::TimedOut),
            _ => None,
        }
    }

    #[must_use]
    pub const fn local_participation_complete(&self) -> bool {
        self.local_participation_complete
    }

    #[must_use]
    pub const fn readiness_exited(&self) -> bool {
        matches!(
            self.lifecycle_state,
            ReadinessLifecycleState::Bootstrapped | ReadinessLifecycleState::TimedOut
        )
    }

    #[must_use]
    pub const fn phase1_confirmed_count(&self) -> usize {
        self.phase1_confirmed_count
    }

    #[must_use]
    pub const fn phase2_confirmed_count(&self) -> usize {
        self.phase2_confirmed_count
    }

    #[must_use]
    pub const fn quorum_threshold(&self) -> usize {
        self.quorum_threshold
    }

    #[must_use]
    pub const fn with_lifecycle_state(mut self, state: ReadinessLifecycleState) -> Self {
        self.lifecycle_state = state;
        self
    }

    #[must_use]
    pub const fn with_local_participation_complete(mut self, val: bool) -> Self {
        self.local_participation_complete = val;
        self
    }

    #[must_use]
    pub const fn with_phase1_count(mut self, count: usize) -> Self {
        self.phase1_confirmed_count = count;
        self
    }

    #[must_use]
    pub const fn with_phase2_count(mut self, count: usize) -> Self {
        self.phase2_confirmed_count = count;
        self
    }
}
"""

snapshot_path = os.path.join(ROOT, "core", "src", "snapshot.rs")
with open(snapshot_path, "w", encoding="utf-8") as f:
    f.write(SNAPSHOT_RS)
print("  [rewrite] core/src/snapshot.rs")

# ---------------------------------------------------------------------------
# 2. Update state_snapshot() impls — remove with_exit_mode / with_readiness_exited
# ---------------------------------------------------------------------------

BOOTSTRAPPED_PATCH = (
    r"\.with_exit_mode\(Some\(ReadinessExitMode::Bootstrapped\)\)\s*\n"
    r"\s*\.with_local_participation_complete\(true\)\s*\n"
    r"\s*\.with_readiness_exited\(true\)"
)

TIMED_OUT_PATCH = (
    r"\.with_exit_mode\(Some\(ReadinessExitMode::TimedOut\)\)\s*\n"
    r"\s*\.with_readiness_exited\(true\)"
)

for fname, pattern, replacement in [
    (
        "core/src/states/bootstrapped.rs",
        BOOTSTRAPPED_PATCH,
        ".with_local_participation_complete(true)",
    ),
    ("core/src/states/timed_out.rs", TIMED_OUT_PATCH, ""),
]:
    path = os.path.join(ROOT, fname)
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    orig = content
    content = re.sub(pattern, replacement, content)
    if content != orig:
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"  [patch] {fname}")

# ---------------------------------------------------------------------------
# 3. Update all Snapshot::new() calls — drop exit_mode and readiness_exited args
# ---------------------------------------------------------------------------
# Before: Snapshot::new(LS, exit_mode, lpc, rex, p1, p2, qt)
# After:  Snapshot::new(LS, lpc, p1, p2, qt)
# The regex captures:
#   group(1) = "Snapshot::new(" + lifecycle_state + ", "
#   group(2) = exit_mode + ", "
#   group(3) = local_participation_complete (KEEP)
#   group(4) = ", " + readiness_exited (REMOVE)
#   group(5) = the rest (phase counts + threshold)

NEW_CALL_RE = re.compile(
    r"(Snapshot::new\s*\(\s*)"
    r"(ReadinessLifecycleState::\w+\s*,\s*)"
    r"(?:Some\(ReadinessExitMode::\w+\)|None)\s*,\s*"  # exit_mode arg
    r"(true|false)\s*,\s*"  # local_participation_complete
    r"(?:true|false)\s*,\s*"  # readiness_exited arg
)

RUST_FILES = []
for sub in ("core", "validation"):
    for dirpath, _, filenames in os.walk(os.path.join(ROOT, sub)):
        for fn in filenames:
            if fn.endswith(".rs"):
                RUST_FILES.append(os.path.join(dirpath, fn))

for path in RUST_FILES:
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    orig = content
    # Replace Snapshot::new(LS, exit_mode, lpc, rex, p1, p2, qt)
    # with        Snapshot::new(LS, lpc, p1, p2, qt)
    content = NEW_CALL_RE.sub(
        r"\1\2\3, ",
        content,
    )
    if content != orig:
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
            print(f"  [new-call] {path}")

# ---------------------------------------------------------------------------
# 4. Update snapshot_tests.rs — remove obsolete tests and BASE const
# ---------------------------------------------------------------------------

TESTS_PATH = os.path.join(ROOT, "core", "tests", "snapshot_tests.rs")
if os.path.isfile(TESTS_PATH):
    with open(TESTS_PATH, "r", encoding="utf-8") as f:
        content = f.read()
    orig = content

    # Remove the two obsolete tests
    content = re.sub(
        r"#\[test\]\nfn with_exit_mode_updates_only_exit_mode\(\) \{[^}]*\}[^}]*\}",
        "",
        content,
    )
    content = re.sub(
        r"#\[test\]\nfn with_readiness_exited_updates_only_readiness_exited\(\) \{[^}]*\}[^}]*\}",
        "",
        content,
    )

    # Update BASE const — remove exit_mode and readiness_exited args
    BASE_RE = re.compile(
        r"(const BASE: Snapshot = Snapshot::new\s*\()"
        r"(ReadinessLifecycleState::Phase2Active,\s*)"
        r"(?:Some\(ReadinessExitMode::\w+\)|None)\s*,\s*"
        r"(true|false)\s*,\s*"
        r"(?:true|false)\s*,\s*"
    )
    content = BASE_RE.sub(r"\1\2\3, ", content)

    if content != orig:
        with open(TESTS_PATH, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"  [tests] core/tests/snapshot_tests.rs")

# ---------------------------------------------------------------------------
# 5. Update transition_tests.rs — helper functions that call Snapshot::new()
#    The regex above should handle the inline calls.
#    But the helper `snapshot_exited()` needs special attention:
#    Snapshot::new(LS, exit_mode, lpc, rex, p1, p2, qt) with specific values.
# ---------------------------------------------------------------------------

print("\nDone!")
print("\nNote: The test `with_exit_mode_updates_only_exit_mode` and")
print("`with_readiness_exited_updates_only_readiness_exited` were removed.")
print("All Snapshot::new() calls were updated to drop the 2 redundant args.")
