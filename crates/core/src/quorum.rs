// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// A node's quorum, as the view shows it: not tracked before the conclusion;
// then held while its members not suspected number the required count or
// more, lost while they number fewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quorum {
    NotTracked,
    Held,
    Lost,
}
