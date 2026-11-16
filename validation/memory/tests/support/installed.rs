// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_memory::counting_allocator::CountingAllocator;

// The counting allocator, installed for the whole test binary: a
// `#[global_allocator]` is one per binary, and every test file measures with
// it.
#[global_allocator]
pub static ALLOCATOR: CountingAllocator = CountingAllocator;
