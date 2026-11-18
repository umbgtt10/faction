// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use core::fmt::Debug;

// What a peer is known by. The machine copies a peer id and asks whether two
// are equal, and nothing more: it never orders one, hashes it or reads its
// value (§1). The unsigned integers are peer ids, u64 every type's default;
// a type of the caller's own is one once it says so, `impl PeerId for Name {}`.
pub trait PeerId: Copy + Eq + Debug + Send {}

impl PeerId for u8 {}

impl PeerId for u16 {}

impl PeerId for u32 {}

impl PeerId for u64 {}

impl PeerId for u128 {}

impl PeerId for usize {}
