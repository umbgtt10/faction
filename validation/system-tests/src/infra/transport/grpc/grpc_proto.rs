// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The code tonic-build writes for faction.proto, namespaced. Its Err variant
// is `tonic::Status` at 176 bytes in every method, and its server trait carries
// a `#[must_use]` the type already has. Neither is ours to change: the file
// this pulls in is rewritten by build.rs on every build, so an allow placed
// inside it would be erased by the next compile.
//
// Scoped to the wrapper rather than the crate, so the same lints still have to
// be answered for anywhere the code is hand-written.
#[allow(clippy::result_large_err, clippy::double_must_use)]
pub mod wire {
    include!("generated/faction.rs");
}
