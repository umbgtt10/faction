// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::command::Command;
use crate::instance::peer_id::PeerId;
use crate::instance::receipt::Receipt;
use crate::observer::Observer;

pub struct NoOpObserver;

impl<P: PeerId> Observer<P> for NoOpObserver {
    fn observe(&mut self, _command: &Command<P>, _receipt: &Receipt<P>) {}
}
