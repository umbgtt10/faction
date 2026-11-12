// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::network::port_allocator::PortAllocator;
use std::net::SocketAddr;
use std::net::TcpListener;

#[test]
fn addr_beyond_the_last_port_saturates() {
    // Arrange
    let ports = PortAllocator::new(Some(65_530));

    // Act
    let addr = ports.addr(10);

    // Assert
    assert_eq!(addr.port(), 65_535);
}

#[test]
fn addr_with_a_base_offsets_it_by_the_index() {
    // Arrange
    let ports = PortAllocator::new(Some(40_000));

    // Act
    let addr = ports.addr(3);

    // Assert
    assert_eq!(addr, "127.0.0.1:40003".parse::<SocketAddr>().unwrap());
}

#[test]
fn addr_without_a_base_is_a_free_loopback_port() {
    // Arrange
    let ports = PortAllocator::new(None);

    // Act
    let addr = ports.addr(0);

    // Assert
    assert!(addr.ip().is_loopback());
    assert_ne!(addr.port(), 0);
    assert!(TcpListener::bind(addr).is_ok());
}
