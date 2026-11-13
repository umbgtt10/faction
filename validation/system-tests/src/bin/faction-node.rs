// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

use std::env::args;
use std::fs::create_dir_all;
use std::io::BufReader;
use std::io::stdin;
use std::io::stdout;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;

use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::quorum_policy::QuorumPolicy;

use faction_protocol::newcomer::Newcomer;
use faction_protocol::protocol::Protocol;
use faction_protocol::transport::Transport;

use faction_system_tests::constants::HELPERS;
use faction_system_tests::faction_node::FactionNode;
use faction_system_tests::infra::logging::shared_file_observer::SharedFileObserver;
use faction_system_tests::infra::logging::shared_writer::SharedWriter;
use faction_system_tests::infra::network::address_book::AddressBook;
use faction_system_tests::infra::timer::real::real_timer::RealTimer;
use faction_system_tests::infra::timer::real::std_monotonic_clock::StdMonotonicClock;
use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::infra::timer::real::thread_pause::ThreadPause;
use faction_system_tests::infra::transport::faulty::fault_policy::FaultPolicy;
use faction_system_tests::infra::transport::faulty::faulty_transport::FaultyTransport;
use faction_system_tests::infra::transport::faulty::link_cuts::LinkCuts;
use faction_system_tests::infra::transport::grpc::grpc_transport::GrpcTransport;
use faction_system_tests::infra::transport::tcp::tcp_transport::TcpTransport;
use faction_system_tests::no_op_node_observer::NoOpNodeObserver;
use faction_system_tests::node_observer::NodeObserver;
use faction_system_tests::process_node::args::Args;
use faction_system_tests::process_node::process_command::ProcessCommand;
use faction_system_tests::process_node::process_runner::ProcessRunner;
use faction_system_tests::role::Role;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::transport_kind::TransportKind;

fn main() {
    let config = Args::new(args().collect()).parse();
    let time = TimePorts {
        clock: Arc::new(StdMonotonicClock::new()),
        pause: Arc::new(ThreadPause),
        wall_clock: Arc::new(SystemWallClock),
    };

    let (observer, node_observer): (Box<dyn Observer>, Box<dyn NodeObserver>) =
        match &config.log_path {
            Some(path) => {
                if let Some(parent) = path.parent() {
                    let _ = create_dir_all(parent);
                }
                let writer = SharedWriter::open(path);
                let observer = Box::new(SharedFileObserver::new(
                    writer.clone(),
                    config.peer_id,
                    time.wall_clock.clone(),
                ));
                let node_observer = Box::new(SharedFileObserver::new(
                    writer,
                    config.peer_id,
                    time.wall_clock.clone(),
                ));
                (observer, node_observer)
            }
            None => (Box::new(NoOpObserver), Box::new(NoOpNodeObserver)),
        };

    let (transport, address_book): (Box<dyn Transport>, AddressBook) = match config.transport {
        TransportKind::Grpc => {
            let transport = GrpcTransport::new(config.listen_addr, &config.peer_addrs);
            let book = transport.registry();
            (Box::new(transport), book)
        }
        TransportKind::Tcp => {
            let transport = TcpTransport::new(
                config.listen_addr,
                config.peer_id,
                &config.peer_addrs,
                time.pause.clone(),
            );
            let book = transport.registry();
            (Box::new(transport), book)
        }
        _ => panic!("unsupported transport for process node"),
    };

    let delay = Duration::from_millis(config.timer_delay_ms);
    let deadline_delay = delay * (config.freshness_margin as u32);
    let timer = Box::new(RealTimer::with_delays(
        time.clock.clone(),
        delay,
        deadline_delay,
        config.probe_stretch,
    ));

    let (peers, role) = if config.newcomer {
        (
            Vec::new(),
            Role::Waiting(Newcomer::new(
                config.peer_id,
                config.suspicion_threshold,
                HELPERS,
                observer,
            )),
        )
    } else {
        let faction_config = Config::new(
            config.peer_id,
            config.peers.clone(),
            QuorumPolicy::new(config.required),
            config.suspicion_threshold,
        );
        let protocol = Protocol::new(
            Faction::new(faction_config, observer),
            config.peers.clone(),
            config.peer_id,
            HELPERS,
        );
        (config.peers, Role::Running(Box::new(protocol)))
    };
    let cuts = LinkCuts::default();
    let faulty = Box::new(FaultyTransport::new(
        config.peer_id,
        FaultPolicy::default(),
        cuts.clone(),
        config.log_path.as_deref().map(SharedWriter::open),
        transport,
    ));
    let node = FactionNode::new(
        config.peer_id,
        peers,
        role,
        faulty,
        timer,
        node_observer,
        cuts,
    );

    let commands = ProcessCommand::read_lines(BufReader::new(stdin()));
    ProcessRunner::new(
        node,
        commands,
        address_book,
        Box::new(stdout()),
        time,
        delay,
    )
    .run();
    exit(0);
}
