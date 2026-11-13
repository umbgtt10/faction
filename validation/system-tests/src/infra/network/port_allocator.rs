// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpListener;

pub struct PortAllocator {
    base: Option<u16>,
}

impl PortAllocator {
    #[must_use]
    pub fn new(base: Option<u16>) -> Self {
        Self { base }
    }

    #[must_use]
    pub fn addr(&self, index: usize) -> SocketAddr {
        match self.base {
            Some(base) => {
                let port = base.saturating_add(u16::try_from(index).unwrap_or(0));
                SocketAddr::from((Ipv4Addr::LOCALHOST, port))
            }
            None => {
                let listener = TcpListener::bind("127.0.0.1:0").unwrap();
                listener.local_addr().unwrap()
            }
        }
    }
}
