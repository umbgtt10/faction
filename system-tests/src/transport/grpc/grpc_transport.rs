// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};

use faction::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::Request;
use tonic::transport::{Channel, Endpoint, Server};

use crate::faction::Envelope;
use crate::faction::transport_client::TransportClient;
use crate::faction::transport_server::TransportServer;
use crate::transport::grpc::grpc_service::GrpcSvc;

type Inbox = Arc<Mutex<VecDeque<TransportMessage>>>;
type Clients = Arc<AsyncMutex<HashMap<PeerId, TransportClient<Channel>>>>;
type Tx = mpsc::UnboundedSender<(PeerId, TransportMessage)>;

pub struct GrpcTransport {
    inbox: Inbox,
    _tx: Tx,
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Drop for GrpcTransport {
    fn drop(&mut self) {
        if let Some(tx) = self._shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl GrpcTransport {
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

    fn spawn_sender(mut rx: mpsc::UnboundedReceiver<(PeerId, TransportMessage)>, clients: Clients) {
        Self::runtime().spawn(async move {
            while let Some((to, msg)) = rx.recv().await {
                let mut guard = clients.lock().await;
                if let Some(c) = guard.get_mut(&to) {
                    let d = Self::encode(&msg);
                    let _ = c.deliver(Request::new(Envelope { data: d })).await;
                }
            }
        });
    }

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

    fn build_server(inbox: Inbox, listen_addr: SocketAddr) -> oneshot::Sender<()> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let rt = Self::runtime();
        let l = std::net::TcpListener::bind(listen_addr).unwrap();
        l.set_nonblocking(true).unwrap();
        let tl = tokio::net::TcpListener::from_std(l).unwrap();
        rt.spawn(async move {
            tokio::select! {
                _ = Server::builder()
                    .add_service(TransportServer::new(GrpcSvc(inbox)))
                    .serve_with_incoming(TcpListenerStream::new(tl)) => {},
                _ = shutdown_rx => {},
            }
        });
        shutdown_tx
    }

    fn build_clients(peer_id: PeerId, peer_addrs: &[(PeerId, SocketAddr)]) -> Clients {
        let rt = Self::runtime();
        let clients: Clients = Arc::new(AsyncMutex::new(HashMap::new()));
        {
            let mut guard = rt.block_on(clients.lock());
            for &(pid, addr) in peer_addrs {
                if pid != peer_id {
                    let ch = rt.block_on(async {
                        Endpoint::from_shared(format!("http://{addr}"))
                            .unwrap()
                            .connect()
                            .await
                            .unwrap()
                    });
                    guard.insert(pid, TransportClient::new(ch));
                }
            }
        }
        clients
    }

    pub fn new(
        listen_addr: SocketAddr,
        peer_id: PeerId,
        peer_addrs: &[(PeerId, SocketAddr)],
    ) -> Self {
        let _guard = Self::runtime().enter();
        let inbox: Inbox = Arc::new(Mutex::new(VecDeque::new()));

        let shutdown_tx = Self::build_server(inbox.clone(), listen_addr);
        let clients = Self::build_clients(peer_id, peer_addrs);

        let (tx, rx) = mpsc::unbounded_channel();
        Self::spawn_sender(rx, clients);

        Self {
            inbox,
            _tx: tx,
            _shutdown_tx: Some(shutdown_tx),
        }
    }

    pub fn new_mesh(peer_ids: &[PeerId]) -> Vec<GrpcTransport> {
        let n = peer_ids.len();
        let rt = Self::runtime();
        let mut addrs = Vec::new();
        let mut inboxes = Vec::new();
        let mut client_lists = Vec::new();
        let mut shutdown_txs = Vec::new();

        {
            let _guard = rt.enter();
            for _ in 0..n {
                let ib: Inbox = Arc::new(Mutex::new(VecDeque::new()));
                let ib_for_server = ib.clone();
                let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
                l.set_nonblocking(true).unwrap();
                let a = l.local_addr().unwrap();
                addrs.push(a);
                let tl = tokio::net::TcpListener::from_std(l).unwrap();
                let (stx, srx) = oneshot::channel();
                rt.spawn(async move {
                    tokio::select! {
                        _ = Server::builder()
                            .add_service(TransportServer::new(GrpcSvc(ib_for_server)))
                            .serve_with_incoming(TcpListenerStream::new(tl)) => {},
                        _ = srx => {},
                    }
                });
                shutdown_txs.push(stx);
                inboxes.push(ib);
            }
        }

        for i in 0..n {
            let clients: Clients = Arc::new(AsyncMutex::new(HashMap::new()));
            {
                let mut guard = rt.block_on(clients.lock());
                for (j, &p) in peer_ids.iter().enumerate() {
                    if i != j {
                        let ch = rt.block_on(async {
                            Endpoint::from_shared(format!("http://{}", addrs[j]))
                                .unwrap()
                                .connect()
                                .await
                                .unwrap()
                        });
                        guard.insert(p, TransportClient::new(ch));
                    }
                }
            }
            client_lists.push(clients);
        }

        inboxes
            .into_iter()
            .zip(client_lists)
            .zip(shutdown_txs)
            .map(|((ib, clients), stx)| {
                let (tx, rx) = mpsc::unbounded_channel();
                Self::spawn_sender(rx, clients);
                GrpcTransport {
                    inbox: ib,
                    _tx: tx,
                    _shutdown_tx: Some(stx),
                }
            })
            .collect()
    }
}

impl Transport for GrpcTransport {
    fn send(&mut self, to: PeerId, m: TransportMessage) {
        let _ = self._tx.send((to, m));
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        self.inbox.lock().unwrap().pop_front()
    }
}
