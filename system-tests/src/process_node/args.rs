// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::env::args;
use std::net::SocketAddr;
use std::path::PathBuf;

use faction::PeerId;

use crate::process_node::config::ProcessNodeConfig;

use crate::transport_kind::TransportKind;

pub struct Args {
    args: Vec<String>,
}

impl Args {
    pub fn from_env() -> Self {
        Self {
            args: args().collect(),
        }
    }

    pub fn parse(&self) -> ProcessNodeConfig {
        let mut peer_id: Option<PeerId> = None;
        let mut peers: Option<Vec<PeerId>> = None;
        let mut required: Option<usize> = None;
        let mut freshness_margin: Option<u64> = None;
        let mut transport: Option<TransportKind> = None;

        let mut listen_addr: Option<SocketAddr> = None;
        let mut peer_addrs: Option<Vec<(PeerId, SocketAddr)>> = None;
        let mut log_path: Option<PathBuf> = None;
        let mut timer_delay_ms: Option<u64> = None;

        let mut i = 1;
        while i < self.args.len() {
            match self.args[i].as_str() {
                "--peer-id" => {
                    i += 1;
                    peer_id = Some(self.args[i].parse().expect("invalid --peer-id"));
                }
                "--peers" => {
                    i += 1;
                    peers = Some(
                        self.args[i]
                            .split(',')
                            .map(|s| s.parse().expect("invalid peer id in --peers"))
                            .collect(),
                    );
                }
                "--required" => {
                    i += 1;
                    required = Some(self.args[i].parse().expect("invalid --required"));
                }
                "--freshness-margin" => {
                    i += 1;
                    freshness_margin =
                        Some(self.args[i].parse().expect("invalid --freshness-margin"));
                }
                "--transport" => {
                    i += 1;
                    transport = Some(match self.args[i].as_str() {
                        "grpc" => TransportKind::Grpc,
                        "tcp" => TransportKind::Tcp,
                        other => panic!("unknown transport: {other}"),
                    });
                }

                "--timer-delay" => {
                    i += 1;
                    timer_delay_ms = Some(self.args[i].parse().expect("invalid --timer-delay"));
                }
                "--listen-addr" => {
                    i += 1;
                    listen_addr = Some(self.args[i].parse().expect("invalid --listen-addr"));
                }
                "--peer-addrs" => {
                    i += 1;
                    peer_addrs = Some(
                        self.args[i]
                            .split(',')
                            .map(|pair| {
                                let (id_str, addr_str) =
                                    pair.split_once('=').expect("invalid --peer-addrs format");
                                let id: PeerId =
                                    id_str.parse().expect("invalid peer id in --peer-addrs");
                                let addr: SocketAddr =
                                    addr_str.parse().expect("invalid address in --peer-addrs");
                                (id, addr)
                            })
                            .collect(),
                    );
                }
                "--log-path" => {
                    i += 1;
                    log_path = Some(PathBuf::from(&self.args[i]));
                }
                other => panic!("unknown argument: {other}"),
            }
            i += 1;
        }

        ProcessNodeConfig {
            peer_id: peer_id.expect("missing --peer-id"),
            peers: peers.expect("missing --peers"),
            required: required.expect("missing --required"),
            freshness_margin: freshness_margin.unwrap_or(2),
            transport: transport.expect("missing --transport"),

            listen_addr: listen_addr.expect("missing --listen-addr"),
            peer_addrs: peer_addrs.expect("missing --peer-addrs"),
            log_path,
            timer_delay_ms: timer_delay_ms.expect("missing --timer-delay"),
        }
    }
}
