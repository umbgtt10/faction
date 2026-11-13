// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    Fresh,
    Pinging,
    Collecting,
    Bootstrapped,
    TimedOut,
}
