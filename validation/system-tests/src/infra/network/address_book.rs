// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

pub type AddressBook = Arc<Mutex<Vec<(u64, SocketAddr)>>>;
