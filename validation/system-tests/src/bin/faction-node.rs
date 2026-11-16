// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

use std::env::args;
use std::fs::create_dir_all;
use std::io::BufReader;
use std::io::stdin;
use std::io::stdout;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;

use faction_protocol::transport::Transport;

use faction_system_tests::faction_node::FactionNode;
use faction_system_tests::infra::fault_board::FaultBoard;
use faction_system_tests::infra::invariants::breaches::Breaches;
use faction_system_tests::infra::live_policy::LivePolicy;
use faction_system_tests::infra::logging::shared_file_observer::SharedFileObserver;
use faction_system_tests::infra::logging::shared_writer::SharedWriter;
use faction_system_tests::infra::network::address_book::AddressBook;
use faction_system_tests::infra::timer::real::real_timer::RealTimer;
use faction_system_tests::infra::timer::real::std_monotonic_clock::StdMonotonicClock;
use faction_system_tests::infra::timer::real::system_wall_clock::SystemWallClock;
use faction_system_tests::infra::timer::real::thread_pause::ThreadPause;
use faction_system_tests::infra::timer::timer_delays::TimerDelays;
use faction_system_tests::infra::transport::faulty::faulty_transport::FaultyTransport;
use faction_system_tests::infra::transport::faulty::link_cuts::LinkCuts;
use faction_system_tests::infra::transport::grpc::grpc_transport::GrpcTransport;
use faction_system_tests::infra::transport::tcp::tcp_transport::TcpTransport;
use faction_system_tests::membership::Membership;
use faction_system_tests::no_op_node_observer::NoOpNodeObserver;
use faction_system_tests::node_observer::NodeObserver;
use faction_system_tests::node_recipe::NodeRecipe;
use faction_system_tests::process_node::args::Args;
use faction_system_tests::process_node::process_command::ProcessCommand;
use faction_system_tests::process_node::process_runner::ProcessRunner;
use faction_system_tests::time_ports::TimePorts;
use faction_system_tests::traits::roles::Roles;
use faction_system_tests::transport_kind::TransportKind;

fn main() {
    let config = Args::new(args().collect()).parse();
    let time = TimePorts {
        clock: Arc::new(StdMonotonicClock::new()),
        pause: Arc::new(ThreadPause),
        wall_clock: Arc::new(SystemWallClock),
    };

    let node_observer: Box<dyn NodeObserver> = match &config.log_path {
        Some(path) => {
            if let Some(parent) = path.parent() {
                let _ = create_dir_all(parent);
            }
            Box::new(SharedFileObserver::new(
                SharedWriter::open(path),
                config.peer_id,
                time.wall_clock.clone(),
            ))
        }
        None => Box::new(NoOpNodeObserver),
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

    let recipe = NodeRecipe::new(
        config.required,
        config.suspicion_threshold,
        TimerDelays {
            delay,
            deadline_delay,
            probe_stretch: config.probe_stretch,
        },
        config
            .log_path
            .as_deref()
            .and_then(Path::parent)
            .map(Path::to_path_buf),
        time.clone(),
        FaultBoard::default(),
        Breaches::default(),
    );
    let role = if config.newcomer {
        recipe.waiting(config.peer_id)
    } else {
        let genesis = Membership {
            peers: config.peers.clone(),
            required_count: config.required,
            epoch: 0,
        };
        recipe.running(config.peer_id, &genesis)
    };
    let cuts = LinkCuts::default();
    let faulty = Box::new(FaultyTransport::new(
        config.peer_id,
        LivePolicy::default(),
        cuts.clone(),
        config.log_path.as_deref().map(SharedWriter::open),
        transport,
    ));
    let node = FactionNode::new(
        config.peer_id,
        role,
        faulty,
        timer,
        node_observer,
        cuts,
        Box::new(recipe),
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
