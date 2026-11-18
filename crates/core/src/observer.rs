// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::instance::peer_id::PeerId;
use crate::instance::receipt::Receipt;

pub trait Observer<P: PeerId = u64> {
    fn observe(&mut self, command: &Command<P>, receipt: &Receipt<P>);
}
