// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use faction::config::Config;
use faction::faction::Faction;
use faction::freshness_policy::FreshnessPolicy;
use faction::no_op_observer::NoOpObserver;
use faction::observer::Observer;
use faction::peer_state::PeerState;
use faction::quorum_policy::QuorumPolicy;

use faction_protocol::protocol::Protocol;
use faction_protocol::timer_trait::Timer;
use faction_protocol::transport_trait::Transport;

use faction_system_tests::faction_node::FactionNode;
use faction_system_tests::no_op_node_observer::NoOpNodeObserver;
use faction_system_tests::node_observer::NodeObserver;
use faction_system_tests::process_node::args::Args;
use faction_system_tests::process_node::run;
use faction_system_tests::timer::in_memory::in_memory_timer::InMemoryTimer;
use faction_system_tests::timer::real::real_timer::RealTimer;
use faction_system_tests::timer_kind::TimerKind;
use faction_system_tests::transport::grpc::grpc_transport::GrpcTransport;
use faction_system_tests::transport::tcp::tcp_transport::TcpTransport;
use faction_system_tests::transport_kind::TransportKind;

fn main() {
    let args = Args::from_env();
    let config = args.parse();

    let observer: Box<dyn Observer> = Box::new(NoOpObserver);
    let node_observer: Box<dyn NodeObserver> = Box::new(NoOpNodeObserver);

    let cfg = Config::new(
        config.peer_id,
        config.peers.clone(),
        QuorumPolicy::new(config.required),
        FreshnessPolicy::new(config.freshness_margin),
    );

    let protocol = Protocol::new(
        Faction::new(cfg, observer),
        config.peers.clone(),
        config.peer_id,
    );

    let transport: Box<dyn Transport> = match config.transport {
        TransportKind::Grpc => Box::new(GrpcTransport::new(
            config.listen_addr,
            config.peer_id,
            &config.peer_addrs,
        )),
        TransportKind::Tcp => Box::new(TcpTransport::new(
            config.listen_addr,
            config.peer_id,
            &config.peer_addrs,
        )),
        _ => panic!("unsupported transport for process node"),
    };

    let timer: Box<dyn Timer> = match config.timer {
        TimerKind::InMemory => Box::new(InMemoryTimer::new()),
        TimerKind::Real => Box::new(RealTimer::new()),
    };

    let node = FactionNode::new(
        config.peer_id,
        config.peers,
        protocol,
        transport,
        timer,
        node_observer,
    );

    let state = run::run(node);
    match state {
        PeerState::Bootstrapped => std::process::exit(0),
        PeerState::TimedOut => std::process::exit(1),
        _ => std::process::exit(2),
    }
}
