// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use faction::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;
use tokio::runtime::Runtime;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::Request;
use tonic::transport::{Channel, Endpoint, Server};

use crate::faction::Envelope;
use crate::faction::transport_client::TransportClient;
use crate::faction::transport_server::TransportServer;
use crate::transport::grpc::grpc_service::GrpcSvc;

type Inbox = Arc<Mutex<VecDeque<TransportMessage>>>;

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
    })
}

pub struct GrpcTransport {
    inbox: Inbox,
    clients: HashMap<PeerId, TransportClient<Channel>>,
}

impl GrpcTransport {
    pub fn new_mesh(peer_ids: &[PeerId]) -> Vec<GrpcTransport> {
        let n = peer_ids.len();
        let rt = runtime();
        let mut addrs = Vec::new();
        let mut inboxes = Vec::new();

        {
            let _guard = rt.enter();
            for _ in 0..n {
                let ib: Inbox = Arc::new(Mutex::new(VecDeque::new()));
                let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
                l.set_nonblocking(true).unwrap();
                let a = l.local_addr().unwrap();
                addrs.push(a);
                let tl = tokio::net::TcpListener::from_std(l).unwrap();
                let ib_clone = ib.clone();
                rt.spawn(async move {
                    Server::builder()
                        .add_service(TransportServer::new(GrpcSvc(ib_clone)))
                        .serve_with_incoming(TcpListenerStream::new(tl))
                        .await
                        .unwrap();
                });
                inboxes.push(ib.clone());
            }
        }

        std::thread::sleep(Duration::from_millis(500));

        inboxes
            .into_iter()
            .enumerate()
            .map(|(i, ib)| {
                let mut cl = HashMap::new();
                for (j, &p) in peer_ids.iter().enumerate() {
                    if i != j {
                        let ch = rt.block_on(async {
                            Endpoint::from_shared(format!("http://{}", addrs[j]))
                                .unwrap()
                                .connect()
                                .await
                                .unwrap()
                        });
                        cl.insert(p, TransportClient::new(ch));
                    }
                }
                GrpcTransport {
                    inbox: ib,
                    clients: cl,
                }
            })
            .collect()
    }

    fn encode(m: &TransportMessage) -> Vec<u8> {
        let (f, t): (PeerId, u8) = match m {
            TransportMessage::Ping { from } => (*from, 0),
            TransportMessage::Ready { from } => (*from, 1),
            TransportMessage::Bootstrapped { from } => (*from, 2),
        };
        let mut d = Vec::with_capacity(9);
        d.extend(&f.to_le_bytes());
        d.push(t);
        d
    }
}

impl Transport for GrpcTransport {
    fn send(&mut self, to: PeerId, m: TransportMessage) {
        if let Some(c) = self.clients.get_mut(&to) {
            let d = Self::encode(&m);
            let _ = runtime().block_on(c.deliver(Request::new(Envelope { data: d })));
        }
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        self.inbox.lock().unwrap().pop_front()
    }
}
